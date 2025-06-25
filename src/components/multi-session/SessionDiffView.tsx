import React, { useEffect, useState } from 'react';
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
  const [diffContent, setDiffContent] = useState<string>('');
  const [isLoading, setIsLoading] = useState(false);

  useEffect(() => {
    loadDiff();
  }, [sessionId]);

  const loadDiff = async () => {
    setIsLoading(true);
    try {
      const stats = await invoke<DiffStats>('get_session_diff', { sessionId });
      setDiffStats(stats);
      
      // For now, we'll just show stats. In a real implementation,
      // we'd also fetch the actual diff content
      setDiffContent(''); // Would be populated with actual diff
    } catch (error) {
      console.error('Failed to load diff:', error);
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="flex flex-col h-full">
      <div className="border-b p-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-4">
            <h3 className="font-medium flex items-center gap-2">
              <GitBranch className="w-4 h-4" />
              Changes
            </h3>
            {diffStats && (
              <div className="flex items-center gap-2 text-sm">
                <FileCode className="w-4 h-4 text-muted-foreground" />
                <span className="text-green-600">+{diffStats.insertions}</span>
                <span className="text-red-600">-{diffStats.deletions}</span>
                <span className="text-muted-foreground">
                  ({diffStats.files_changed} files)
                </span>
              </div>
            )}
          </div>
          <Button
            variant="outline"
            size="sm"
            onClick={loadDiff}
            disabled={isLoading}
          >
            <RefreshCw className={`w-4 h-4 ${isLoading ? 'animate-spin' : ''}`} />
          </Button>
        </div>
      </div>

      <ScrollArea className="flex-1 p-4">
        {diffStats && diffStats.files_changed === 0 ? (
          <div className="text-center text-muted-foreground py-8">
            No changes in this session yet
          </div>
        ) : (
          <div className="font-mono text-sm">
            <div className="bg-muted p-4 rounded-md">
              <div className="space-y-2">
                <div>Files changed: {diffStats?.files_changed || 0}</div>
                <div className="text-green-600">
                  Insertions: +{diffStats?.insertions || 0}
                </div>
                <div className="text-red-600">
                  Deletions: -{diffStats?.deletions || 0}
                </div>
              </div>
            </div>
            {diffContent && (
              <pre className="mt-4 whitespace-pre-wrap">{diffContent}</pre>
            )}
          </div>
        )}
      </ScrollArea>
    </div>
  );
}