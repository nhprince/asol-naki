/**
 * Typed contracts for every Rust command we invoke.
 * Keep these in sync with src-tauri/src — serde does the rest.
 */

export interface BasicHardwareInfo {
  cpu_name: string;
  cpu_threads: number;
  total_memory_mb: number;
  os_name: string;
  os_version: string;
  kernel_version: string;
  hostname: string;
}

export interface FullHardwareInfo extends BasicHardwareInfo {
  cpu_cores_physical?: number;
  motherboard?: string;
  bios_vendor?: string;
  bios_version?: string;
  gpus?: { name: string; vram_bytes?: number; driver_version?: string }[];
}

export interface BatteryInfo {
  design_capacity_mwh?: number;
  full_charge_capacity_mwh?: number;
  health_percent?: number;
  cycle_count?: number;
  manufacturer?: string;
  chemistry?: string;
}

export interface StorageInfo {
  model_name?: string;
  serial?: string;
  protocol?: string;
  total_capacity_bytes?: number;
  sector_size?: number;
  smart_status?: string;
  nvme_percentage_used?: number;
  realloc_sector_count?: number;
  pending_sector_count?: number;
  media_errors?: number;
  power_on_hours?: number;
  temperature_c?: number;
}

export type Verdict = "walk-away" | "negotiate" | "good-buy";
