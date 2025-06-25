export interface SessionInfo {
  id: string;
  project_id: string;
  project_path: string;
  worktree_path: string;
  branch_name: string;
  status: SessionStatus;
  created_at: string;
  updated_at: string;
  auto_yes: boolean;
  output_preview: string;
  diff_stats?: DiffStats;
}

export type SessionStatus = 
  | 'initializing'
  | 'running'
  | 'ready'
  | 'loading'
  | 'paused'
  | 'error'
  | 'completed'
  | 'terminated';

export interface SessionConfig {
  auto_yes: boolean;
  max_output_buffer: number;
  environment_vars: Array<[string, string]>;
  working_directory: string | null;
  branch_prefix: string;
  claude_args: string[];
}

export interface DiffStats {
  files_changed: number;
  insertions: number;
  deletions: number;
}

export type SessionEvent = 
  | { type: 'StatusChanged'; data: { session_id: string; status: SessionStatus } }
  | { type: 'OutputAppended'; data: { session_id: string; output: string } }
  | { type: 'DiffUpdated'; data: { session_id: string; stats: DiffStats } }
  | { type: 'SessionCreated'; data: { session_id: string } }
  | { type: 'SessionTerminated'; data: { session_id: string } }
  | { type: 'Error'; data: { session_id: string; error: string } };