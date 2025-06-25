import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Button } from '@/components/ui/button';
import { Play, Pause, Square } from 'lucide-react';
import { SessionInfo } from '@/types/multi-session';

interface SessionControlsProps {
  session: SessionInfo;
  onUpdate?: () => void;
}

export default function SessionControls({ session, onUpdate }: SessionControlsProps) {
  const [, setToast] = useState<{ message: string; type: "success" | "error" | "info" } | null>(null);

  const handlePause = async () => {
    try {
      await invoke('pause_session', { sessionId: session.id });
      setToast({
        message: `Session ${session.id.slice(0, 8)} has been paused`,
        type: 'success',
      });
      onUpdate?.();
    } catch (error) {
      setToast({
        message: String(error),
        type: 'error',
      });
    }
  };

  const handleResume = async () => {
    try {
      await invoke('resume_session', { sessionId: session.id });
      setToast({
        message: `Session ${session.id.slice(0, 8)} has been resumed`,
        type: 'success',
      });
      onUpdate?.();
    } catch (error) {
      setToast({
        message: String(error),
        type: 'error',
      });
    }
  };

  const handleTerminate = async () => {
    if (!confirm(`Are you sure you want to terminate session ${session.id.slice(0, 8)}?`)) {
      return;
    }

    try {
      await invoke('terminate_session', { sessionId: session.id });
      setToast({
        message: `Session ${session.id.slice(0, 8)} has been terminated`,
        type: 'success',
      });
      onUpdate?.();
    } catch (error) {
      setToast({
        message: String(error),
        type: 'error',
      });
    }
  };

  const canPause = session.status === 'running' || session.status === 'ready';
  const canResume = session.status === 'paused';
  const canTerminate = session.status !== 'terminated';

  return (
    <div className="flex items-center gap-2">
      {canPause && (
        <Button
          variant="outline"
          size="sm"
          onClick={handlePause}
          title="Pause session"
        >
          <Pause className="w-4 h-4" />
        </Button>
      )}
      
      {canResume && (
        <Button
          variant="outline"
          size="sm"
          onClick={handleResume}
          title="Resume session"
        >
          <Play className="w-4 h-4" />
        </Button>
      )}
      
      {canTerminate && (
        <Button
          variant="outline"
          size="sm"
          onClick={handleTerminate}
          title="Terminate session"
          className="text-destructive"
        >
          <Square className="w-4 h-4" />
        </Button>
      )}
    </div>
  );
}