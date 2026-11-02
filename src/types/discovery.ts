export interface DiscoveryQuery {
  search: string;
  page_token: string | null;
  order: "name" | "name_desc" | "modified_desc" | "modified_asc";
  drive_id: string | null;
  page_size: number;
}
export interface DiscoveredFile {
  id: string;
  name: string;
  mime_type: string;
  modified_time: string | null;
  shared: boolean;
  drive_id: string | null;
  resource_key: string | null;
}
export interface DiscoveryPage {
  files: DiscoveredFile[];
  next_page_token: string | null;
  incomplete_search: boolean;
  complete: boolean;
  corpus: string;
}
