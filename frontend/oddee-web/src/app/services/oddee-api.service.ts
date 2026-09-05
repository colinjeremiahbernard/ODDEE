import { Injectable } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Observable } from 'rxjs';

export interface PhysicalEvent {
  id: string;
  occurred_at: string;
  entity_id: string;
  kind: string;
  zone: string;
  source: string;
  metadata: Record<string, unknown>;
}

export interface Anomaly {
  id: string;
  detected_at: string;
  severity: string;
  score: number;
  title: string;
  explanation: string;
  status: string;
  related_event_ids: string[];
}

export interface ListEnvelope<T> {
  items: T[];
  total: number;
  limit: number;
  offset: number;
  has_more: boolean;
}

const API_BASE = 'http://localhost:3000';

@Injectable({
  providedIn: 'root',
})
export class OddeeApiService {
  constructor(private http: HttpClient) {}

  getEvents(params?: {
    entity_id?: string;
    zone?: string;
    limit?: number;
    offset?: number;
  }): Observable<ListEnvelope<PhysicalEvent>> {
    return this.http.get<ListEnvelope<PhysicalEvent>>(`${API_BASE}/events`, { params });
  }

  getAnomalies(params?: {
    severity?: string;
    status?: string;
    limit?: number;
    offset?: number;
  }): Observable<ListEnvelope<Anomaly>> {
    return this.http.get<ListEnvelope<Anomaly>>(`${API_BASE}/anomalies`, { params });
  }
}
