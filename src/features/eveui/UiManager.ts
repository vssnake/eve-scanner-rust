import {invoke} from "@tauri-apps/api/core";
import {listen} from "@tauri-apps/api/event";
import {UiStatus} from "./UiModel.ts";
import {plainToInstance} from "class-transformer";
import {useUiEventStore} from "./uiEventStore.ts";

export class UiManager {
  private uiEventStore = useUiEventStore();

  constructor() {
      listen<any>('eve_ui_status', (event) => {

          if (typeof event.payload.general_window === 'string') {
              event.payload.general_window = JSON.parse(event.payload.general_window);
          }
          const rootObject: UiStatus = plainToInstance(UiStatus, event.payload);

          console.log('RootObject:', rootObject);

          this.uiEventStore.onUiUpdated(rootObject);

      }).then((_) => {
          //TODO: Unlisten
      });
  }
  
  
    public startTracker(pid: number) {
        invoke('start_tracker', { pid } )
            .then((response) => {
                console.log("Respuesta desde Rust:", response);
            })
            .catch((error) => {
                console.error("Error invocando el comando en Rust:", error);
            });
    }
    
    public stopTracker(pid: number) {
      
        invoke('stop_tracker', { pid } )
            .then((response) => {
                console.log("Respuesta desde Rust:", response);
            })
            .catch((error) => {
                console.error("Error invocando el comando en Rust:", error);
            });
    }
}