import {UiStatus} from "./UiModel.ts";

export interface UiEvent {
    onUiUpdate(uiStatus: UiStatus): void;
}