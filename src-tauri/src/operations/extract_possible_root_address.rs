use std::sync::Mutex;
use crate::process::interop::memory::memory_utils::transform_memory_content_as_ulong_memory;
use rayon::prelude::*;
use crate::process::interop::memory::windows_memory_reader::WindowsMemoryReader;

pub struct ExtractPossibleRootAddress {
    memory_reader: Option<WindowsMemoryReader>,
}

impl ExtractPossibleRootAddress {
    pub fn new() -> Box<ExtractPossibleRootAddress> {
        Box::new(ExtractPossibleRootAddress {
            memory_reader: None,
        })
    }

    pub fn execute(&mut self, process_id: u32) -> Result<Vec<u64>, String> {
        self.memory_reader = WindowsMemoryReader::new(process_id);

        let memory_regions = self.memory_reader.as_ref().unwrap().read_commited_region();

        let mut ordered_memory_regions: Vec<_> = memory_regions
            .into_iter()
            .map(|region| (region.base_address, region.length))
            .collect();

        ordered_memory_regions.sort_by_key(|region| region.0);

        let cloned_memory_regions = ordered_memory_regions.clone();

        let candidates: Vec<_> = cloned_memory_regions
            .into_par_iter()
            .flat_map(|region| {
                self.enumerate_candidates_for_python_type_object_type_in_memory_region(
                    region,
                    &WindowsMemoryReader::new(process_id).unwrap(),
                )
            })
            .collect();

        let ui_root_type_object_candidates: Vec<_> = self
            .enumerate_candidates_for_python_type_objects(
                &ordered_memory_regions,
                &candidates,
                &WindowsMemoryReader::new(process_id).unwrap(),
            )
            .into_iter()
            .filter(|type_object| type_object.1 == "UIRoot")
            .map(|type_object| type_object.0)
            .collect();

        let candidates = self.enumerate_candidates_for_instances_of_python_type(
            &ui_root_type_object_candidates,
            &ordered_memory_regions,
            process_id,
        );

        Ok(candidates)
    }

    fn enumerate_candidates_for_python_type_object_type_in_memory_region(
        &self,
        memory_region: (u64, u64),
        windows_memory_reader: &WindowsMemoryReader,
    ) -> Vec<u64> {
        let memory_content =
            self.read_memory_region_content_as_ulong_array(memory_region, &windows_memory_reader);

        if memory_content.is_none() {
            return Vec::new();
        }

        let memory_content = memory_content.unwrap();
        let mut result = Vec::new();

        for i in 0..memory_content.len() - 4 {
            let candidate_address = memory_region.0 + (i as u64) * 8;
            let candidate_type = memory_content[i + 1];

            if candidate_type != candidate_address {
                continue;
            }

            if let Some(candidate_name) = self
                .read_null_terminated_ascii_string_from_address_up_to255(
                    memory_content[i + 3],
                    &windows_memory_reader,
                )
            {
                if candidate_name == "type" {
                    result.push(candidate_address);
                }
            }
        }

        result
    }

    fn enumerate_candidates_for_python_type_objects(
        &self,
        memory_regions: &[(u64, u64)],
        type_object_candidates: &[u64],
        windows_memory_reader: &WindowsMemoryReader,
    ) -> Vec<(u64, String)> {
        if type_object_candidates.is_empty() {
            return Vec::new();
        }

        let type_address_min = *type_object_candidates.iter().min().unwrap();
        let type_address_max = *type_object_candidates.iter().max().unwrap();

        let result = Mutex::new(Vec::new());

        memory_regions.into_par_iter().for_each(|&memory_region| {
            let memory_content = self
                .read_memory_region_content_as_ulong_array(memory_region, windows_memory_reader);

            if memory_content.is_none() {
                return;
            }

            let memory_content = memory_content.unwrap();

            for i in 0..memory_content.len() - 4 {
                let candidate_address = memory_region.0 + (i as u64) * 8;
                let candidate_type = memory_content[i + 1];

                if candidate_type < type_address_min || candidate_type > type_address_max {
                    continue;
                }

                if !type_object_candidates.contains(&candidate_type) {
                    continue;
                }

                if let Some(candidate_name) = self
                    .read_null_terminated_ascii_string_from_address_up_to255(
                        memory_content[i + 3],
                        windows_memory_reader,
                    )
                {
                    result.lock().unwrap().push((candidate_address, candidate_name));
                }
            }
        });

        /*for &memory_region in memory_regions {
            let memory_content = self
                .read_memory_region_content_as_ulong_array(memory_region, windows_memory_reader);

            if memory_content.is_none() {
                continue;
            }

            let memory_content = memory_content.unwrap();

            for i in 0..memory_content.len() - 4 {
                let candidate_address = memory_region.0 + (i as u64) * 8;
                let candidate_type = memory_content[i + 1];

                if candidate_type < type_address_min || candidate_type > type_address_max {
                    continue;
                }

                if !type_object_candidates.contains(&candidate_type) {
                    continue;
                }

                if let Some(candidate_name) = self
                    .read_null_terminated_ascii_string_from_address_up_to255(
                        memory_content[i + 3],
                        windows_memory_reader,
                    )
                {
                    result.push((candidate_address, candidate_name));
                }
            }
        }*/

        result.into_inner().unwrap()
    }

    fn enumerate_candidates_for_instances_of_python_type(
        &self,
        type_object_candidates: &[u64],
        memory_regions: &[(u64, u64)],
        process_id: u32,
    ) -> Vec<u64> {
        if type_object_candidates.is_empty() {
            return Vec::new();
        }

        let type_address_min = *type_object_candidates.iter().min().unwrap();
        let type_address_max = *type_object_candidates.iter().max().unwrap();

        let result = Mutex::new(Vec::new());

        memory_regions.into_par_iter().for_each(|&memory_region| {
            let windows_memory_reader = WindowsMemoryReader::new(process_id).unwrap();
            let memory_content = self
                .read_memory_region_content_as_ulong_array(memory_region, &windows_memory_reader);

            if memory_content.is_none() {
                return;
            }

            let memory_content = memory_content.unwrap();

            for i in 0..memory_content.len() - 4 {
                let candidate_address = memory_region.0 + (i as u64) * 8;
                let candidate_type = memory_content[i + 1];

                if candidate_type < type_address_min || candidate_type > type_address_max {
                    continue;
                }

                if type_object_candidates.contains(&candidate_type) {
                    result.lock().unwrap().push(candidate_address);
                }
            }
        });
        /*for &memory_region in memory_regions {
            let memory_content = self
                .read_memory_region_content_as_ulong_array(memory_region, windows_memory_reader);

            if memory_content.is_none() {
                continue;
            }

            let memory_content = memory_content.unwrap();

            for i in 0..memory_content.len() - 4 {
                let candidate_address = memory_region.0 + (i as u64) * 8;
                let candidate_type = memory_content[i + 1];

                if candidate_type < type_address_min || candidate_type > type_address_max {
                    continue;
                }

                if type_object_candidates.contains(&candidate_type) {
                    result.push(candidate_address);
                }
            }
        }*/

        result.into_inner().unwrap()
    }

    fn read_memory_region_content_as_ulong_array(
        &self,
        memory_region: (u64, u64),
        windows_memory_reader: &WindowsMemoryReader,
    ) -> Option<Vec<u64>> {
        let byte_array = windows_memory_reader.read_bytes(memory_region.0, memory_region.1);
        
        if byte_array.is_err() {
            return None;
        }
        
        let result =byte_array.unwrap();
        
        Some(transform_memory_content_as_ulong_memory(&result))
    }

    fn read_null_terminated_ascii_string_from_address_up_to255(
        &self,
        address: u64,
        windows_memory_reader: &WindowsMemoryReader,
    ) -> Option<String> {
        let memory = windows_memory_reader.read_bytes(address, 0x100);
        
        if memory.is_err() {
            return None;
        }

        let mut length = 0;
        
        let memory_result = memory.unwrap();
        for (i, &byte) in memory_result.iter().enumerate() {
            if byte == 0 {
                length = i;
                break;
            }
        }

        Some(String::from_utf8_lossy(&memory_result[..length]).to_string())
    }
}
