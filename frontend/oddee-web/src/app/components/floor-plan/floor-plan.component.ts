import {
  ChangeDetectionStrategy,
  Component,
  Input,
  OnChanges,
  SimpleChanges,
} from '@angular/core';
import { CommonModule } from '@angular/common';

import { Anomaly, PhysicalEvent } from '../../services/oddee-api.service';

interface ZoneLayout {
  id: string;
  label: string;
  /** SVG path "d" attribute for the zone outline. */
  shape: 'rect' | 'polygon';
  x?: number;
  y?: number;
  width?: number;
  height?: number;
  points?: string;
  /** Description shown in tooltip / legend. */
  description: string;
}

interface ZoneEventCount {
  zone: string;
  events: number;
  anomalies: number;
  hasAnomaly: boolean;
  lastKind: string | null;
  lastEntity: string | null;
}

@Component({
  selector: 'app-floor-plan',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './floor-plan.component.html',
  styleUrls: ['./floor-plan.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class FloorPlanComponent implements OnChanges {
  @Input() events: PhysicalEvent[] = [];
  @Input() anomalies: Anomaly[] = [];

  /**
   * Static warehouse layout. Coordinates are in a 1000x560 SVG viewport.
   * Zones are taken from the README's warehouse scenario and
   * the event metadata (`zone` field, e.g. "warehouse-a").
   */
  readonly zones: ZoneLayout[] = [
    {
      id: 'loading-dock',
      label: 'Loading dock',
      shape: 'rect',
      x: 30,
      y: 30,
      width: 220,
      height: 120,
      description: 'Inbound and outbound staging.',
    },
    {
      id: 'warehouse-a',
      label: 'Zone A — Static shelving',
      shape: 'rect',
      x: 280,
      y: 30,
      width: 240,
      height: 220,
      description: 'Packages that should not move without a workflow.',
    },
    {
      id: 'warehouse-b',
      label: 'Zone B — Active shelving',
      shape: 'rect',
      x: 550,
      y: 30,
      width: 220,
      height: 220,
      description: 'Frequently restocked inventory.',
    },
    {
      id: 'warehouse-c',
      label: 'Zone C — Bulk storage',
      shape: 'rect',
      x: 800,
      y: 30,
      width: 170,
      height: 220,
      description: 'Pallets and oversized goods.',
    },
    {
      id: 'packing',
      label: 'Packing',
      shape: 'rect',
      x: 30,
      y: 180,
      width: 220,
      height: 160,
      description: 'Order assembly and labeling.',
    },
    {
      id: 'dispatch',
      label: 'Dispatch',
      shape: 'rect',
      x: 30,
      y: 370,
      width: 220,
      height: 160,
      description: 'Outbound staging and loading.',
    },
    {
      id: 'routes',
      label: 'Forklift routes',
      shape: 'polygon',
      points: '280,270 770,270 770,530 280,530',
      description: 'Approved movement corridors.',
    },
  ];

  readonly aisleMarkers: { x: number; y: number; label: string }[] = [
    { x: 380, y: 100, label: 'A-12' },
    { x: 460, y: 100, label: 'A-13' },
    { x: 380, y: 170, label: 'A-14' },
    { x: 460, y: 170, label: 'A-15' },
    { x: 620, y: 110, label: 'B-04' },
    { x: 700, y: 110, label: 'B-05' },
    { x: 620, y: 180, label: 'B-06' },
    { x: 700, y: 180, label: 'B-07' },
    { x: 855, y: 110, label: 'C-01' },
    { x: 920, y: 110, label: 'C-02' },
  ];

  zoneStats: Record<string, ZoneEventCount> = {};
  totalAnomalousZones = 0;
  totalEventMarkers = 0;

  ngOnChanges(_changes: SimpleChanges): void {
    this.zoneStats = this.computeZoneStats();
    this.totalAnomalousZones = Object.values(this.zoneStats).filter(
      (z) => z.hasAnomaly,
    ).length;
    this.totalEventMarkers = Object.values(this.zoneStats).reduce(
      (sum, z) => sum + z.events,
      0,
    );
  }

  private computeZoneStats(): Record<string, ZoneEventCount> {
    const stats: Record<string, ZoneEventCount> = {};
    for (const zone of this.zones) {
      stats[zone.id] = {
        zone: zone.id,
        events: 0,
        anomalies: 0,
        hasAnomaly: false,
        lastKind: null,
        lastEntity: null,
      };
    }

    const anomalousEventIds = new Set<string>();
    for (const anomaly of this.anomalies) {
      for (const id of anomaly.related_event_ids ?? []) {
        anomalousEventIds.add(id);
      }
    }

    for (const event of this.events) {
      const zoneId = this.normalizeZone(event.zone);
      if (!stats[zoneId]) {
        stats[zoneId] = {
          zone: zoneId,
          events: 0,
          anomalies: 0,
          hasAnomaly: false,
          lastKind: null,
          lastEntity: null,
        };
      }
      stats[zoneId].events += 1;
      stats[zoneId].lastKind = event.kind;
      stats[zoneId].lastEntity = event.entity_id;
      if (anomalousEventIds.has(event.id)) {
        stats[zoneId].anomalies += 1;
        stats[zoneId].hasAnomaly = true;
      }
    }

    return stats;
  }

  /**
   * Map the free-form `zone` string from the API to a known zone id.
   * The seed data uses names like "warehouse-a" and "shelf-b-12";
   * collapse those to a canonical zone id.
   */
  private normalizeZone(zone: string | null | undefined): string {
    const z = (zone ?? '').toLowerCase();
    if (!z) return 'loading-dock';
    if (z.startsWith('warehouse-a') || z.startsWith('shelf-a')) return 'warehouse-a';
    if (z.startsWith('warehouse-b') || z.startsWith('shelf-b')) return 'warehouse-b';
    if (z.startsWith('warehouse-c') || z.startsWith('shelf-c')) return 'warehouse-c';
    if (z.startsWith('pack')) return 'packing';
    if (z.startsWith('dispatch') || z.startsWith('outbound')) return 'dispatch';
    if (z.startsWith('dock') || z.startsWith('loading')) return 'loading-dock';
    if (z.startsWith('route')) return 'routes';
    return z;
  }

  /** Generate deterministic marker positions inside a zone for event pulses. */
  pulsePositions(zoneId: string, count: number): { x: number; y: number }[] {
    const zone = this.zones.find((z) => z.id === zoneId);
    if (!zone || count === 0) return [];
    const positions: { x: number; y: number }[] = [];
    const max = Math.min(count, 8);
    if (zone.shape === 'rect' && zone.x != null && zone.y != null) {
      const w = zone.width ?? 0;
      const h = zone.height ?? 0;
      for (let i = 0; i < max; i++) {
        const seed = this.hash(zoneId + ':' + i);
        const x = zone.x + 30 + (seed % Math.max(1, w - 60));
        const y = zone.y + 30 + ((seed >> 8) % Math.max(1, h - 60));
        positions.push({ x, y });
      }
    }
    return positions;
  }

  private hash(s: string): number {
    let h = 0;
    for (let i = 0; i < s.length; i++) {
      h = (h * 31 + s.charCodeAt(i)) >>> 0;
    }
    return h;
  }

  trackZone(_i: number, zone: ZoneLayout): string {
    return zone.id;
  }

  trackPulse(_i: number, p: { x: number; y: number }): string {
    return `${p.x}:${p.y}`;
  }

  trackAisle(_i: number, a: { x: number; y: number; label: string }): string {
    return a.label;
  }
}
