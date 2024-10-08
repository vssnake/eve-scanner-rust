﻿use crate::eve::interop::memory::memory_reading_cache::MemoryReadingCache;
use crate::eve::interop::memory::models::dict_entry_representation::PyDictEntryRepresentation;
use crate::eve::interop::memory::python_memory_reader::PythonMemoryReader;
use crate::eve::interop::memory::utils::MemoryUtils;
use crate::eve::interop::memory::windows_memory_reader::WindowsMemoryReader;
use crate::eve::interop::memory::python_type_extractor::PythonTypeExtractor;
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use crate::eve::ui_tree_node::models::child_of_node::{ChildWithRegion, ChildWithoutRegion};
use crate::eve::ui_tree_node::models::display_region::DisplayRegion;
use crate::eve::ui_tree_node::models::ui_tree_node::{UITreeNodeWithDisplayRegion, UiTreeNode};
use crate::eve::ui_tree_node::ui_constants::{UiConstants, UiZonesEnum, UI_ZONES};
use crate::eve::ui_tree_node::utils::display_region_utils::DisplayRegionUtils;

pub struct UiTreeNodeExtractor {
    windows_memory_reader_ext: PythonMemoryReader,
    memory_reader: Rc<WindowsMemoryReader>,
    memory_reading_cache: MemoryReadingCache,
}

impl UiTreeNodeExtractor {
    pub fn new(process_id: u32) -> UiTreeNodeExtractor {
        let memory_reader = Rc::new(WindowsMemoryReader::new(process_id).unwrap());

        UiTreeNodeExtractor {
            windows_memory_reader_ext: PythonMemoryReader::new(&memory_reader),
            memory_reader: Rc::clone(&memory_reader),
            memory_reading_cache: MemoryReadingCache::new(),
        }
    }

    pub fn extract_ui_tree_from_address(
        &self,
        address: u64,
        max_depth: i32,
    ) -> Result<(Rc<UITreeNodeWithDisplayRegion>,HashMap<UiZonesEnum, Vec<Rc<UITreeNodeWithDisplayRegion>>>), &'static str> {
        self.memory_reading_cache.clear();
        let children_with_zones: RefCell<
            HashMap<UiZonesEnum, Vec<Rc<UITreeNodeWithDisplayRegion>>>,
        > = RefCell::new(UiConstants::initialize_mapper());
        
        let node = self.read_ui_tree_from_address(address, max_depth, None, None, &children_with_zones);
        node.map(|node| (node,  children_with_zones.into_inner()))
    }

    fn read_ui_tree_from_address(
        &self,
        node_address: u64,
        max_depth: i32,
        total_display_region: Option<Rc<DisplayRegion>>,
        occluded_regions: Option<Vec<Rc<DisplayRegion>>>,
        children_with_zones: &RefCell<HashMap<UiZonesEnum, Vec<Rc<UITreeNodeWithDisplayRegion>>>>,
    ) -> Result<Rc<UITreeNodeWithDisplayRegion>, &'static str> {
        //let mut cache = cache.unwrap_or_else(MemoryReadingCache::new);
        let ui_node_memory = self.memory_reader.read_bytes(node_address, 0x30)?;

        if ui_node_memory.len() != 0x30 {
            return Err("Node memory is not 0x30 bytes long");
        }

        let python_type_name = self
            .windows_memory_reader_ext
            .get_python_type_name_from_object_address(node_address, &self.memory_reading_cache)?;

        if python_type_name.is_empty() {
            return Err("Failed to read python type name");
        }
        //let test = &ui_node_memory[0x10..];
        let dict_address = u64::from_le_bytes(ui_node_memory[0x10..0x18].try_into().unwrap());
        //let dict_address = u64::from_le_bytes(ui_node_memory[0x10..].try_into().unwrap());
        let dictionary_entries = self
            .windows_memory_reader_ext
            .read_active_dictionary_entries_from_dictionary_address(dict_address)?;

        let mut dict_entries_of_interest: HashMap<String, Rc<Box<dyn Any>>> = HashMap::new();
        let mut other_dict_entries_keys = Vec::new();

        for entry in dictionary_entries.iter() {
            let key_type_name = self
                .windows_memory_reader_ext
                .get_python_type_name_from_object_address(entry.key, &self.memory_reading_cache)?;

            if key_type_name.as_str() != "str" {
                continue;
            }

            let key_string = self
                .windows_memory_reader_ext
                .read_python_string_value_max_length_4000(entry.key, &self.memory_reading_cache)?;

            if !PythonTypeExtractor::is_key_of_interest(&key_string) {
                other_dict_entries_keys.push(key_string);
                continue;
            }

            let dict_entry_value = self
                .windows_memory_reader_ext
                .get_dict_entry_value_representation(entry.value, &self.memory_reading_cache);

            /*if (matches!(&dict_entry_value, _DictEntryValueGenericRepresentation)) {
                continue;
            }*/

            let generic_value_representation = dict_entry_value
                .as_ref()
                .downcast_ref::<PyDictEntryRepresentation>();

            if generic_value_representation.is_some() {
                if let Some(object_type_name) = &generic_value_representation
                    .unwrap()
                    .python_object_type_name
                {
                    if object_type_name == "NoneType" {
                        continue;
                    }
                }
            }

            if (key_string == "_display") {
                let is_visible = if let Some(boolean) = dict_entry_value.downcast_ref::<bool>() {
                    *boolean
                } else {
                    false
                };

                if is_visible == false {
                    return Err("Display is false");
                }
            }

            /*let dict_entry_representation =  self.windows_memory_reader_ext
            .get_dict_entry_value_representation(entry.value,&self.memory_reading_cache);*/

            dict_entries_of_interest.insert(key_string, dict_entry_value);
        }

        let self_display_region =
            DisplayRegionUtils::get_display_region_from_dict_entries(&dict_entries_of_interest)
                .unwrap_or_else(|| DisplayRegion::new(0, 0, 0, 0));

        let cloned_self_display_region = Rc::new(self_display_region);

        let total_display_region =
            total_display_region.unwrap_or(Rc::clone(&cloned_self_display_region));
        let mut occluded_regions = occluded_regions.unwrap_or_else(Vec::new);

        let (children, childs_with_region, childs_without_region, total_display_region_visible) =
            self.read_childrens(
                node_address,
                max_depth,
                &dict_entries_of_interest,
                Rc::clone(&total_display_region),
                &mut occluded_regions,
                children_with_zones,
            )
            .unwrap_or_else(|_| {
                (
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    DisplayRegion::new(0, 0, 0, 0),
                )
            });

        let ui_tree_node = UiTreeNode::new(
            node_address,
            python_type_name,
            dict_entries_of_interest,
            other_dict_entries_keys,
            children,
        );

        let node_with_display_region = UITreeNodeWithDisplayRegion {
            ui_node: Rc::new(ui_tree_node),
            child_with_region: childs_with_region,
            child_without_region: childs_without_region,
            self_display_region: Rc::clone(&cloned_self_display_region),
            total_display_region: Rc::clone(&total_display_region),
            total_display_region_visible,
        };

        let node = Rc::new(node_with_display_region);
        // Add the node to the corresponding zone
        /*if let Some(zone) = UI_ZONES.get(&node.ui_node.object_type_name.as_str()) {
            children_with_zones
                .borrow_mut()
                .entry(zone.clone())
                .or_insert_with(Vec::new)
                .push(Rc::clone(&node));
        }*/

        UiConstants::check_and_insert_inportant_zone(children_with_zones, &node.ui_node.object_type_name, Rc::clone(&node));

        Ok(Rc::clone(&node))
    }

    fn read_childrens(
        &self,
        node_address: u64,
        max_depth: i32,
        dict_entries_of_interest: &HashMap<String, Rc<Box<dyn Any>>>,
        total_display_region: Rc<DisplayRegion>,
        occluded_regions: &mut Vec<Rc<DisplayRegion>>,
        children_with_zones: &RefCell<HashMap<UiZonesEnum, Vec<Rc<UITreeNodeWithDisplayRegion>>>>,
    ) -> Result<
        (
            Vec<Rc<UiTreeNode>>,
            Vec<Rc<ChildWithRegion>>,
            Vec<Rc<ChildWithoutRegion>>,
            DisplayRegion,
        ),
        &'static str,
    > {
        //  https://github.com/Arcitectus/Sanderling/blob/b07769fb4283e401836d050870121780f5f37910/guide/image/2015-01.eve-online-python-ui-tree-structure.png

        let child_addresses =
            self.get_children_addresses(node_address, dict_entries_of_interest)?;

        let mut children_tree_nodes: Vec<Rc<UiTreeNode>> = Vec::new();
        let mut childs_with_region: Vec<Rc<ChildWithRegion>> = Vec::new();
        let mut childs_without_region: Vec<Rc<ChildWithoutRegion>> = Vec::new();
        let mut occluded_regions_from_siblings = Vec::new();

        for child_address in child_addresses {
            let child_result = self.read_ui_tree_from_address(
                child_address,
                max_depth - 1,
                Some(Rc::clone(&total_display_region)),
                Some(occluded_regions.clone()),
                children_with_zones,
            );
            
            if child_result.is_err() {
                continue;
            }
            let child = child_result.unwrap();

            let pointer_total_display_region = &total_display_region;
            let child_result = DisplayRegionUtils::create_display_region_node_with_offset(
                (
                    pointer_total_display_region.x,
                    pointer_total_display_region.y,
                ),
                &mut occluded_regions_from_siblings,
                &child.ui_node,
            );

            let child_with_region_option =
                DisplayRegionUtils::just_case_with_display_region(Rc::clone(&child_result));

            if (child_with_region_option.is_some()) {
                let child_with_region = Rc::clone(&child_with_region_option.unwrap());
                let descendants_with_display_region =
                    DisplayRegionUtils::list_descendants_with_display_region(
                        &child_with_region.node.child_with_region,
                    );

                occluded_regions_from_siblings.extend(
                    descendants_with_display_region
                        .into_iter()
                        .filter(|child_w_region| {
                            DisplayRegionUtils::node_occludes_following_nodes(
                                (&child_w_region.node),
                            )
                        })
                        .map(|child_w_region| Rc::clone(&child_w_region.node.total_display_region)),
                );

                childs_with_region.insert(0, child_with_region); // Insert at the start to build the list in reverse order

                occluded_regions.extend(occluded_regions_from_siblings.iter().cloned());
            } else {
                childs_without_region.insert(
                    0,
                    child_result
                        .as_any_rc()
                        .downcast::<ChildWithoutRegion>()
                        .unwrap(),
                ); // Insert at the start to build the list in reverse order
            }

            children_tree_nodes.push(Rc::clone(&child.ui_node));
        }

        childs_with_region.reverse(); // Reverse to correct the order after processing
        childs_without_region.reverse();

        let total_display_region_visible = DisplayRegion::new(-1, -1, 0, 0);

        Ok((
            children_tree_nodes,
            childs_with_region,
            childs_without_region,
            total_display_region_visible,
        ))
    }

    fn get_children_addresses(
        &self,
        node_address: u64,
        dict_entries_of_interest: &HashMap<String, Rc<Box<dyn Any>>>,
    ) -> Result<Vec<u64>, &'static str> {
        let children_dict_entry = dict_entries_of_interest.get("children");

        if children_dict_entry.is_none() {
            return Err("Not found children key in dict entries of interest");
        }
        
        if !children_dict_entry.unwrap().is::<PyDictEntryRepresentation>() {
            return Err("Children entry is not a PyDictEntryRepresentation");
        }

        let children_entry_object_address = children_dict_entry
            .unwrap()
            .downcast_ref::<PyDictEntryRepresentation>()
            .unwrap()
            .address;

        let py_children_list_memory = self
            .memory_reader
            .read_bytes(children_entry_object_address, 0x18)?;

        if py_children_list_memory.len() != 0x18 {
            return Err("Children list memory is not 0x18 bytes long");
        }

        let py_children_dict_address =
            u64::from_le_bytes(py_children_list_memory[0x10..].try_into().unwrap());
        let py_children_dict_entries = self
            .windows_memory_reader_ext
            .read_active_dictionary_entries_from_dictionary_address(py_children_dict_address)?;

        let children_entry = py_children_dict_entries.into_iter().find(|entry| {
            let key_type_name = self
                .windows_memory_reader_ext
                .get_python_type_name_from_object_address(entry.key, &self.memory_reading_cache);
            if key_type_name.map_or(false, |name| name != "str") {
                return false;
            }
            let key_string_result = self
                .windows_memory_reader_ext
                .read_python_string_value_max_length_4000(entry.key, &self.memory_reading_cache);
            if key_string_result.is_err() {
                return false;
            }
            let key_string = key_string_result.unwrap();

            return key_string == "_childrenObjects";
            //key_string.map_or(false, |s| s == "_childrenObjects")
        });

        if children_entry.is_none() {
            return Err("Not found _childrenObjects key in children dict entries");
        }

        let python_list_object_memory = self
            .memory_reader
            .read_bytes(children_entry.unwrap().value, 0x20)?;

        if python_list_object_memory.len() != 0x20 {
            return Err("Python list object memory is not 0x20 bytes long");
        }

        let bytes_slice = &python_list_object_memory[0x10..0x18];

        let list_ob_size = u64::from_le_bytes(bytes_slice.try_into().unwrap());

        if list_ob_size > 4000 {
            return Err("List ob size is greater than 4000");
        }

        let list_entries_size = (list_ob_size * 8) as usize;
        let list_ob_item =
            u64::from_le_bytes(python_list_object_memory[0x18..].try_into().unwrap());

        let list_entries_memory = self
            .memory_reader
            .read_bytes(list_ob_item, list_entries_size as u64)?;

        Ok(
            MemoryUtils::transform_memory_content_as_ulong_memory(&list_entries_memory)
                .into_iter()
                .collect(),
        )
    }
}
