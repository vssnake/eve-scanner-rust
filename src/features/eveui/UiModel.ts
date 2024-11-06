import { Expose, Type } from 'class-transformer';
import 'reflect-metadata';

class IconSpriteColorPercent {
    @Expose({ name: 'alpha' })
    alpha!: number;

    @Expose({ name: 'red' })
    red!: number;

    @Expose({ name: 'green' })
    green!: number;

    @Expose({ name: 'blue' })
    blue!: number;
}

class CommonIndications {
    @Expose({ name: 'targeting' })
    targeting!: boolean;

    @Expose({ name: 'targeted_by_me' })
    targetedByMe!: boolean;

    @Expose({ name: 'is_jamming_me' })
    isJammingMe!: boolean;

    @Expose({ name: 'is_warp_disrupting_me' })
    isWarpDisruptingMe!: boolean;
}

class CellsTexts {
    @Expose({ name: 'Nombre' })
    nombre!: string;

    @Expose({ name: 'Distancia' })
    distancia!: string;
}

export class OverviewWindowEntry {
    @Expose({ name: 'texts_left_to_right' })
    textsLeftToRight!: string[];

    @Expose({ name: 'cells_texts' })
    @Type(() => CellsTexts)
    cellsTexts!: CellsTexts;

    @Expose({ name: 'object_distance' })
    objectDistance!: string;

    @Expose({ name: 'object_distance_in_meters' })
    objectDistanceInMeters!: number | null;

    @Expose({ name: 'object_name' })
    objectName!: string;

    @Expose({ name: 'object_type' })
    objectType!: string | null;

    @Expose({ name: 'object_alliance' })
    objectAlliance!: string | null;

    @Expose({ name: 'is_player' })
    isPlayer!: boolean;

    @Expose({ name: 'icon_sprite_color_percent' })
    @Type(() => IconSpriteColorPercent)
    iconSpriteColorPercent!: IconSpriteColorPercent;

    @Expose({ name: 'names_under_space_object_icon' })
    namesUnderSpaceObjectIcon!: string[];

    @Expose({ name: 'bg_color_fills_percent' })
    bgColorFillsPercent!: any[];

    @Expose({ name: 'right_aligned_icons_hints' })
    rightAlignedIconsHints!: any[];

    @Expose({ name: 'common_indications' })
    @Type(() => CommonIndications)
    commonIndications!: CommonIndications;

    @Expose({ name: 'opacity_percent' })
    opacityPercent!: number | null;
}

class OverviewWindow {
    @Expose({ name: 'entries' })
    @Type(() => OverviewWindowEntry)
    entries!: OverviewWindowEntry[];
}

class DirectionalScannerEntry {
    @Expose({ name: 'distance' })
    distance!: number | null;

    @Expose({ name: 'names' })
    names!: string;

    @Expose({ name: 'ship_type' })
    shipType!: string;

    @Expose({ name: 'ship_icon' })
    shipIcon!: string;
}

class DirectionalScanner {
    @Expose({ name: 'entries' })
    @Type(() => DirectionalScannerEntry)
    entries!: DirectionalScannerEntry[];
}

class ProbeScannerEntry {
    @Expose({ name: 'distance_unformatted' })
    distanceUnformatted!: string;

    @Expose({ name: 'distance' })
    distance!: number | null;

    @Expose({ name: 'id' })
    id!: string;

    @Expose({ name: 'name' })
    name!: string;

    @Expose({ name: 'signal_strength' })
    signalStrength!: string;

    @Expose({ name: 'type_emplacement' })
    typeEmplacement!: string;
}

class ProbeScanner {
    @Expose({ name: 'entries' })
    @Type(() => ProbeScannerEntry)
    entries!: ProbeScannerEntry[];
}

export class RootObject {
    @Expose({ name: 'overview_windows' })
    @Type(() => OverviewWindow)
    overviewWindows!: OverviewWindow[];

    @Expose({ name: 'directional_scanner' })
    @Type(() => DirectionalScanner)
    directionalScanner!: DirectionalScanner;

    @Expose({ name: 'probe_scanner' })
    @Type(() => ProbeScanner)
    probeScanner!: ProbeScanner;
}

export class UiStatus {
    @Expose({ name: 'general_window' })
    @Type(() => RootObject)
    rootObject!: RootObject;
    
    @Expose({ name: 'ms_processing' })
    msProcessing!: number;
    
    @Expose({ name: 'status' })
    status!: string;
    
    @Expose({ name: 'process_id'})
    processId!: number
    
    @Expose({ name: 'error'})
    error!: number
}
