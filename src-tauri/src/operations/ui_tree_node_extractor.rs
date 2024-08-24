use std::any::Any;
use std::collections::HashMap;
use std::rc::Rc;
use crate::eve::ui::common::common::{ChildOfNodeWithDisplayRegion, DisplayRegion, UITreeNodeWithDisplayRegion};
use crate::eve::ui::ui_constants::{UiConstants, UiZonesEnum, UI_ZONES};
use crate::process;
use crate::process::interop::memory::eve_python_utils::{as_ui_tree_node_with_inherited_offset, get_display_region_from_dict_entries, just_case_with_display_region, list_descendants_with_display_region, EvePythonUtils, NodeOcclusion};
use crate::process::interop::memory::memory_reading_cache::MemoryReadingCache;
use crate::process::interop::memory::memory_utils::transform_memory_content_as_ulong_memory;
use crate::process::interop::memory::object_type::ObjectType;
use crate::process::interop::memory::python_memory_reader::PythonMemoryReader;
use crate::process::interop::memory::python_models::{DictEntry, DictEntryValueGenericRepresentation, PyDictEntry};
use crate::process::interop::memory::windows_memory_reader::WindowsMemoryReader;
use crate::process::interop::ui::ui_tree_node::UiTreeNode;

pub struct UiTreeNodeExtractor<'a> {
    windows_memory_reader_ext: PythonMemoryReader,
    memory_reader: Rc<WindowsMemoryReader>,
    children_with_zones: &'a mut HashMap<UiZonesEnum, Vec<Rc<UITreeNodeWithDisplayRegion>>>,
}

impl UiTreeNodeExtractor<'_> {

    pub fn new(memory_reader: &Rc<WindowsMemoryReader>) -> UiTreeNodeExtractor {

        UiTreeNodeExtractor {
            windows_memory_reader_ext: PythonMemoryReader::new(memory_reader, MemoryReadingCache::new()),
            memory_reader: Rc::clone(memory_reader),
            children_with_zones: &mut UiConstants::initialize_mapper(),
        }
    }

    pub fn extract_ui_tree_from_address(&self, address: u64, max_depth: i32) -> Option<Rc<UITreeNodeWithDisplayRegion>> {
        self.read_ui_tree_from_address(address, max_depth, None, None)
    }

    fn read_ui_tree_from_address(
        &self,
        node_address: u64,
        max_depth: i32,
        total_display_region: Option<&DisplayRegion>,
        occluded_regions: Option<Vec<DisplayRegion>>,
    ) -> Option<Rc<UITreeNodeWithDisplayRegion>> {
        //let mut cache = cache.unwrap_or_else(MemoryReadingCache::new);
        let ui_node_memory = self.memory_reader.read_bytes(node_address, 0x30)?;

        if ui_node_memory.len() != 0x30 {
            return None;
        }

        let python_type_name = self.windows_memory_reader_ext.get_python_type_name_from_object_address(node_address)?;

        if python_type_name.is_empty() {
            return None;
        }

        let dict_address = u64::from_le_bytes(ui_node_memory[0x10..].try_into().unwrap());
        let dictionary_entries = self.windows_memory_reader_ext.read_active_dictionary_entries_from_dictionary_address(dict_address)?;

        let mut dict_entries_of_interest : HashMap<String,Box<dyn Any>>  = HashMap::new();
        let mut other_dict_entries_keys = Vec::new();

        for entry in dictionary_entries.iter() {
            let key_type_name = self.windows_memory_reader_ext.get_python_type_name_from_object_address(entry.key)?;

            if key_type_name.as_str() != "str" {
                continue;
            }

            let key_string = self.windows_memory_reader_ext.read_python_string_value_max_length_4000(entry.key)?;

            if !EvePythonUtils::is_key_of_interest(&key_string) {
                other_dict_entries_keys.push(key_string);
                continue;
            }

            let dict_entry_value = self.windows_memory_reader_ext.get_dict_entry_value_representation(entry.value)?;

            if (matches!(&dict_entry_value, _DictEntryValueGenericRepresentation)) {
                continue;
            }


            if let Some(DictEntryValueGenericRepresentation { python_object_type_name: Some(ref name), .. }) = dict_entry_value.as_ref().downcast_ref() {
                if **name == "NoneType" {
                    continue;
                }
            }

            if (key_string == "_display") {
                let result = if let Some(boolean) = dict_entry_value.downcast_ref::<bool>() {
                    *boolean == false
                } else {
                    false
                };

                if result == false {
                    return None;
                }
            }
            dict_entries_of_interest.insert(
                key_string,
                Box::new(self.windows_memory_reader_ext.get_dict_entry_value_representation(entry.value)));

        }

        let self_display_region = get_display_region_from_dict_entries(&dict_entries_of_interest)
            .unwrap_or_else(|| DisplayRegion::new(0, 0, 0, 0));

        let cloned_self_display_region = self_display_region.clone();

        let mut total_display_region = total_display_region.unwrap_or(&self_display_region);
        let mut occluded_regions = occluded_regions.unwrap_or_else(Vec::new);

        let (children, mapped_siblings, total_display_region_visible) = self.read_childrens(
            node_address,
            max_depth,
            &dict_entries_of_interest,
            &mut total_display_region,
            &mut occluded_regions,
        )?;

        let ui_tree_node = UiTreeNode::new(
            node_address,
            python_type_name,
            dict_entries_of_interest,
            other_dict_entries_keys,
            children,
        );


        let node_with_display_region = UITreeNodeWithDisplayRegion {

            ui_node: ui_tree_node,
            children: Some(mapped_siblings),
            self_display_region: cloned_self_display_region,
            total_display_region: total_display_region.clone(),
            total_display_region_visible,
        };

        let node = Rc::new(node_with_display_region);

        // Add the node to the corresponding zone
        if let Some(zone) = UI_ZONES.get(node_with_display_region.ui_node.object_type_name.as_str()) {
            &self.children_with_zones.entry(zone).or_insert_with(Vec::new).push(&node);
        }

        Some(node)
    }

    fn read_childrens(
        &self,
        node_address: u64,
        max_depth: i32,
        dict_entries_of_interest: &HashMap<String, Box<dyn Any>>,
        total_display_region: &mut DisplayRegion,
        occluded_regions: &mut Vec<DisplayRegion>,
    ) -> Option<(Vec<&Rc<UiTreeNode>>, Vec<Rc<dyn ChildOfNodeWithDisplayRegion>>, DisplayRegion)> {


        if max_depth < 1 {
            return None;
        }

        //  https://github.com/Arcitectus/Sanderling/blob/b07769fb4283e401836d050870121780f5f37910/guide/image/2015-01.eve-online-python-ui-tree-structure.png


        let child_addresses = self.get_children_addresses(node_address, dict_entries_of_interest)?;

        let mut children_tree_nodes = Vec::new();
        let mut mapped_siblings = Vec::new();
        let mut occluded_regions_from_siblings = Vec::new();

        for child_address in child_addresses {
            let child = self.read_ui_tree_from_address(child_address, max_depth - 1, Some(total_display_region), Some(occluded_regions.clone()))?;

            let child_result = as_ui_tree_node_with_inherited_offset(
                (total_display_region.x, total_display_region.y),
                &mut occluded_regions_from_siblings,
                &child.ui_node,
            );

            let child_with_region = just_case_with_display_region(child_result.as_ref());
            let descendants_with_display_region = list_descendants_with_display_region(child_with_region?.node.children.as_ref());

            occluded_regions_from_siblings.extend(
                descendants_with_display_region
                    .into_iter()
                    .filter(|child_w_region| NodeOcclusion::node_occludes_following_nodes((&child_w_region.node)))
                    .map(|child_w_region| child_w_region.node.total_display_region),
            );

            mapped_siblings.insert(0, child_result); // Insert at the start to build the list in reverse order
            occluded_regions.extend(occluded_regions_from_siblings.iter().cloned());
            children_tree_nodes.push(&child.ui_node);
        }

        mapped_siblings.reverse(); // Reverse to correct the order after processing

        let total_display_region_visible = DisplayRegion::new(-1, -1, 0, 0);

        Some((children_tree_nodes, mapped_siblings, total_display_region_visible))
    }

    fn get_children_addresses(
        &self,
        node_address: u64,
        dict_entries_of_interest: &HashMap<String, Box<dyn Any>>,
    ) -> Option<Vec<u64>> {
        let children_dict_entry = dict_entries_of_interest.get("children")?;

        let children_entry_object_address = children_dict_entry.downcast_ref::<DictEntryValueGenericRepresentation>()?.address;


        let py_children_list_memory = self.memory_reader.read_bytes(children_entry_object_address, 0x18)?;

        if py_children_list_memory.len() != 0x18 {
            return None;
        }

        let py_children_dict_address = u64::from_le_bytes(py_children_list_memory[0x10..].try_into().unwrap());
        let py_children_dict_entries = self.windows_memory_reader_ext.read_active_dictionary_entries_from_dictionary_address(py_children_dict_address)?;

        let children_entry = py_children_dict_entries.into_iter().find(|entry| {
            let key_type_name = self.windows_memory_reader_ext.get_python_type_name_from_object_address(entry.key)?;
            if key_type_name != "str" {
                return false;
            }
            let key_string = self.windows_memory_reader_ext.read_python_string_value_max_length_4000(entry.key)?;
            key_string == "_childrenObjects"
        })?;

        let python_list_object_memory = self.memory_reader.read_bytes(children_entry.value, 0x20)?;

        if python_list_object_memory.len() != 0x20 {
            return None;
        }

        let list_ob_size = u64::from_le_bytes(python_list_object_memory[0x10..].try_into().unwrap());

        if list_ob_size > 4000 {
            return None;
        }

        let list_entries_size = (list_ob_size * 8) as usize;
        let list_ob_item = u64::from_le_bytes(python_list_object_memory[0x18..].try_into().unwrap());

        let list_entries_memory = self.memory_reader.read_bytes(list_ob_item, list_entries_size as u64)?;

        Some(
            transform_memory_content_as_ulong_memory(list_entries_memory)
                .into_iter()
                .collect(),
        )
    }
}