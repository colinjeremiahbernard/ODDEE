import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { forkJoin, of } from 'rxjs';
import { catchError } from 'rxjs/operators';
import { FloorPlanComponent } from '../../components/floor-plan/floor-plan.component';

import { OddeeApiService,
  PhysicalEvent,
  Anomaly,
  ListEnvelope } from '../../services/oddee-api.service';

const NEXT_STATUS: Record<string, { label: string; next: string }[]> = {
  new: [
    { label: 'Investigate', next: 'investigating' },
    { label: 'Dismiss', next: 'dismissed' },
  ],
  investigating: [
    { label: 'Resolve', next: 'resolved' },
    { label: 'Dismiss', next: 'dismissed' },
  ],
  resolved: [{ label: 'Reopen', next: 'new' }],
  dismissed: [{ label: 'Reopen', next: 'new' }],
};

@Component({
  selector: 'app-dashboard',
  standalone: true,
  imports: [CommonModule, FormsModule, FloorPlanComponent],
  templateUrl: './dashboard.component.html',
  styleUrls: ['./dashboard.component.scss'],
})
export class DashboardComponent implements OnInit {
  events: PhysicalEvent[] = [];
  anomalies: Anomaly[] = [];
  loading = true;
  error: string | null = null;

  /** IDs of anomaly cards currently expanded to show full details + evidence. */
  expandedIds = new Set<string>();
  /** Evidence events already fetched for an expanded anomaly, keyed by anomaly id. */
  evidenceByAnomaly: Record<string, PhysicalEvent[]> = {};
  evidenceLoading = new Set<string>();
  /** Anomaly ids currently mid status-update, to disable their buttons. */
  statusUpdating = new Set<string>();

  constructor(private api: OddeeApiService) {}

  ngOnInit(): void {
    this.load();
  }

  load(): void {
    this.loading = true;
    this.error = null;

    forkJoin({
      events: this.api.getEvents({ limit: 20 }),
      anomalies: this.api.getAnomalies({ limit: 20 }),
    }).subscribe({
      next: ({ events, anomalies }) => {
        this.events = events.items ?? [];
        this.anomalies = anomalies.items ?? [];
        this.loading = false;
      },
      error: (err) => {
        console.error('Dashboard load failed:', err);
        this.error = 'Failed to load dashboard data';
        this.loading = false;
      },
    });
  }

  statusActions(anomaly: Anomaly): { label: string; next: string }[] {
    return NEXT_STATUS[anomaly.status] ?? [];
  }

  isExpanded(anomaly: Anomaly): boolean {
    return this.expandedIds.has(anomaly.id);
  }

  toggleDetails(anomaly: Anomaly): void {
    if (this.expandedIds.has(anomaly.id)) {
      this.expandedIds.delete(anomaly.id);
      return;
    }

    this.expandedIds.add(anomaly.id);

    if (
      this.evidenceByAnomaly[anomaly.id] ||
      !anomaly.related_event_ids ||
      anomaly.related_event_ids.length === 0
    ) {
      return;
    }

    this.evidenceLoading.add(anomaly.id);
    forkJoin(
      anomaly.related_event_ids.map((id) =>
        this.api.getEvent(id).pipe(catchError(() => of(null))),
      ),
    ).subscribe((events) => {
      this.evidenceByAnomaly[anomaly.id] = events.filter(
        (e): e is PhysicalEvent => e !== null,
      );
      this.evidenceLoading.delete(anomaly.id);
    });
  }

  updateStatus(anomaly: Anomaly, next: string): void {
    this.statusUpdating.add(anomaly.id);
    this.api.updateAnomalyStatus(anomaly.id, next).subscribe({
      next: (updated) => {
        anomaly.status = updated.status;
        this.statusUpdating.delete(anomaly.id);
      },
      error: (err) => {
        console.error('Failed to update anomaly status:', err);
        this.statusUpdating.delete(anomaly.id);
      },
    });
  }
}
