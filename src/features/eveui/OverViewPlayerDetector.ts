import {OverviewWindowEntry} from "./UiModel.ts";
import {useUiEventStore} from "./uiEventStore.ts";
import {watch} from "vue";
import {UiEvent} from "./UiEvent.ts";

export class OverViewPlayerDetector implements UiEvent {

    private whitelistPlayers: string[] = [];
    private readonly onlyPlayers: boolean = false;
    private oldEntries: OverviewWindowEntry[] = [];
    private lastAlertTime: number | null = null;

    private uiEventStore = useUiEventStore();


    constructor(whitelistPlayers: string[], onlyPlayers: boolean) {
        this.whitelistPlayers = whitelistPlayers;
        this.onlyPlayers = onlyPlayers;
        this.onUiUpdate()
    }

    public onUiUpdate(): void {

        watch(this.uiEventStore.overViewEntries, (newMap: Map<number, OverviewWindowEntry[]>, _: Map<number, OverviewWindowEntry[]>) => {
            const test2 = newMap.values();

            const entries = Array.from(test2).flatMap<OverviewWindowEntry>((entries: OverviewWindowEntry[]) => {
                return entries.filter((entry) => {
                    if (!this.whitelistPlayers.includes(entry.objectName)) {
                        return !(this.onlyPlayers && !entry.isPlayer);
                    }
                });
            });

            if (!this.areArraysEqualByObjectNameAndType(entries, this.oldEntries)) {
                this.alertEnemyPlayer().then(_ => console.log("Alerted"));
                console.log("New entries:", entries);
            }

            this.oldEntries = entries;
        });

    };

    private async alertEnemyPlayer(){
        const now = Date.now();
        if (!this.lastAlertTime || (now - this.lastAlertTime) > 10000) {

            const audio = new Audio('/sounds/reaper.mp3');
            await audio.play();

            this.lastAlertTime = now;
        }

        console.log("Enemy player detected");
    }

    private areArraysEqualByObjectNameAndType(
        array1: OverviewWindowEntry[],
        array2: OverviewWindowEntry[]
    ): boolean {
        if (array1.length !== array2.length) {
            return false;
        }

        const map1 = new Map(array1.map(item => [item.objectName + item.objectType, item]));
        const map2 = new Map(array2.map(item => [item.objectName + item.objectType, item]));

        for (const key of map1.keys()) {
            if (!map2.has(key)) {
                return false;
            }
        }

        return true;
    }

}