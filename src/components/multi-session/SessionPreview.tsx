import React, { useEffect, useState, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Send, Download, Trash2 } from 'lucide-react';
import { SessionInfo } from '@/types/multi-session';

interface SessionPreviewProps {
  session: SessionInfo;
}

export default function SessionPreview({ session }: SessionPreviewProps) {
  const [output, setOutput] = useState<string[]>([]);
  const [input, setInput] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [toast, setToast] = useState<{ message: string; type: "success" | "error" | "info" } | null>(null);
  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    loadOutput();
  }, [session.id]);

  useEffect(() => {
    // Auto-scroll to bottom when new output arrives
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [output]);

  const loadOutput = async () => {
    try {
      const lines = await invoke<string[]>('get_session_output', {
        sessionId: session.id,
        lines: 1000,
      });
      setOutput(lines);
    } catch (error) {
      console.error('Failed to load output:', error);
    }
  };

  const sendInput = async () => {
    if (!input.trim()) return;
    
    setIsLoading(true);
    try {
      await invoke('send_input', {
        sessionId: session.id,
        input: input.trim(),
      });
      setInput('');
    } catch (error) {
      setToast({
        message: String(error),
        type: 'error',
      });
    } finally {
      setIsLoading(false);
    }
  };

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      sendInput();
    }
  };

  const exportOutput = () => {
    const content = output.join('\n');
    const blob = new Blob([content], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `session-${session.id.slice(0, 8)}-output.txt`;
    a.click();
    URL.revokeObjectURL(url);
  };

  const clearOutput = () => {
    setOutput([]);
  };

  const isInputEnabled = session.status === 'ready' || session.status === 'running';

  return (
    <div className="flex flex-col h-full">
      <div className="border-b p-2 flex items-center justify-between">
        <div className="text-sm text-muted-foreground">
          Session Output - {session.worktree_path}
        </div>
        <div className="flex items-center gap-2">
          <Button
            variant="ghost"
            size="sm"
            onClick={exportOutput}
            title="Export output"
          >
            <Download className="w-4 h-4" />
          </Button>
          <Button
            variant="ghost"
            size="sm"
            onClick={clearOutput}
            title="Clear output"
          >
            <Trash2 className="w-4 h-4" />
          </Button>
        </div>
      </div>

      <ScrollArea className="flex-1 p-4" ref={scrollRef}>
        <div className="font-mono text-sm space-y-1">
          {output.map((line, index) => (
            <div key={index} className="whitespace-pre-wrap break-all">
              {line}
            </div>
          ))}
          {output.length === 0 && (
            <div className="text-muted-foreground text-center py-8">
              No output yet. The session will display output here as it runs.
            </div>
          )}
        </div>
      </ScrollArea>

      {isInputEnabled && (
        <div className="border-t p-4">
          <div className="flex items-center gap-2">
            <Input
              value={input}
              onChange={(e) => setInput(e.target.value)}
              onKeyPress={handleKeyPress}
              placeholder="Type a command..."
              disabled={!isInputEnabled || isLoading}
              className="flex-1"
            />
            <Button
              onClick={sendInput}
              disabled={!isInputEnabled || isLoading || !input.trim()}
              size="sm"
            >
              <Send className="w-4 h-4" />
            </Button>
          </div>
        </div>
      )}
    </div>
  );
}