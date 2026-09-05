import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { forkJoin } from 'rxjs';

import {
  OddeeApiService,
  PhysicalEvent,
  Anomaly,
} from '../../services/oddee-api.service';

@Component({
  selector: 'app-dashboard',
  standalone: true,
  imports: [CommonModule, FormsModule],
  templateUrl: './dashboard.component.html',
  styleUrls: ['./dashboard.component.scss'],
})
export class DashboardComponent implements OnInit {
  events: PhysicalEvent[] = [];
  anomalies: Anomaly[] = [];

  loading = true;
  error: string | null = null;

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
        console.log('Events:', events);
        console.log('Anomalies:', anomalies);

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
}
