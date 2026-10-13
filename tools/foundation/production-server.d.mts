export interface ProductionServer {
  url: string;
  build: { sha256: string; files: number; source: string; scope: string };
  close(): Promise<void>;
}
export function startProductionServer(options?: { directory?: string; port?: number }): Promise<ProductionServer>;
