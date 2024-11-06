import {defineStore} from "pinia";
import {ref} from "vue";
import {OverviewWindowEntry, UiStatus} from "./UiModel.ts";

export const useUiEventStore = defineStore('uiEvents', () => {
    const actualOverViewEntries = ref<Map<number, OverviewWindowEntry[]>>(new Map<number, OverviewWindowEntry[]>());
    

    function onUiUpdated(uiStatus: UiStatus) {
        const overViewEntries = 
            uiStatus.rootObject.overviewWindows.flatMap((overviewWindow) => {
            return overviewWindow.entries;
        });

        actualOverViewEntries.value.set(uiStatus.processId, overViewEntries);
    }
    
    return {
        overViewEntries: actualOverViewEntries,
        onUiUpdated

    }
})