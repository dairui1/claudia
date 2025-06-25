import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Button } from '@/components/ui/button';
import { RefreshCw, GitBranch, FileCode } from 'lucide-react';

import { DiffStats } from '@/types/multi-session';

interface SessionDiffViewProps {
  sessionId: string;
}

export default function SessionDiffView({ sessionId }: SessionDiffViewProps) {
  const [diffStats, setDiffStats] = useState<DiffStats | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadDiffStats();
  }, [sessionId]);

  const loadDiffStats = async () => {
    setLoading(true);
    setError(null);
    try {
      const stats = await invoke<DiffStats>('get_session_diff', {
        sessionId,
      });
      setDiffStats(stats);
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="flex flex-col h-full">
      <div className="border-b p-4 flex items-center justify-between">
        <div className="flex items-center gap-2">
          <GitBranch className="w-4 h-4" />
          <h3 className="font-medium">Changes</h3>
        </div>
        <Button
          variant="ghost"
          size="sm"
          onClick={loadDiffStats}
          disabled={loading}
        >
          <RefreshCw className={`w-4 h-4 ${loading ? 'animate-spin' : ''}`} />
        </Button>
      </div>

      <ScrollArea className="flex-1 p-4">
        {loading && (
          <div className="flex items-center justify-center py-8">
            <div className="animate-spin rounded-full h-6 w-6 border-b-2 border-primary"></div>
          </div>
        )}

        {error && (
          <div className="text-center py-8">
            <p className="text-destructive text-sm">{error}</p>
            <Button
              variant="outline"
              size="sm"
              onClick={loadDiffStats}
              className="mt-2"
            >
              Retry
            </Button>
          </div>
        )}

        {!loading && !error && diffStats && (
          <div className="space-y-4">
            <div className="grid grid-cols-3 gap-4">
              <div className="text-center p-4 border rounded-lg">
                <FileCode className="w-6 h-6 mx-auto mb-2 text-muted-foreground" />
                <div className="text-2xl font-bold">{diffStats.files_changed}</div>
                <div className="text-sm text-muted-foreground">Files Changed</div>
              </div>
              
              <div className="text-center p-4 border rounded-lg">
                <div className="text-2xl font-bold text-green-600">+{diffStats.insertions}</div>
                <div className="text-sm text-muted-foreground">Insertions</div>
              </div>
              
              <div className="text-center p-4 border rounded-lg">
                <div className="text-2xl font-bold text-red-600">-{diffStats.deletions}</div>
                <div className="text-sm text-muted-foreground">Deletions</div>
              </div>
            </div>

            {diffStats.files_changed === 0 && (
              <div className="text-center py-8 text-muted-foreground">
                No changes detected in this session.
              </div>
            )}
          </div>
        )}

        {!loading && !error && !diffStats && (
          <div className="text-center py-8 text-muted-foreground">
            Unable to load diff statistics.
          </div>
        )}
      </ScrollArea>
    </div>
  );
}