import {invoke} from '@tauri-apps/api/core';


export async function getEveProcess(): Promise<number[]> {
    try {
        console.log('getEveProcess');
        return await invoke<number[]>('get_process_ids');
    } catch (error) {
        await Promise.reject(error);
    }
}