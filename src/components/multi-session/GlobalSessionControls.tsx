import React from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { PauseCircle, PlayCircle, StopCircle, Zap } from 'lucide-react';
import { SessionInfo } from '@/types/multi-session';
import { useToast } from '@/components/ui/use-toast';

interface GlobalSessionControlsProps {
  sessions: SessionInfo[];
}

export default function GlobalSessionControls({ sessions }: GlobalSessionControlsProps) {
  const { toast } = useToast();

  const sessionCounts = sessions.reduce((acc, session) => {
    acc.total++;
    acc[session.status] = (acc[session.status] || 0) + 1;
    return acc;
  }, { total: 0 } as Record<string, number>);

  const pauseAll = async () => {
    const runningSessions = sessions.filter(s => 
      s.status === 'running' || s.status === 'ready'
    );

    if (runningSessions.length === 0) {
      toast({
        title: 'No sessions to pause',
        description: 'All sessions are already paused or terminated',
      });
      return;
    }

    try {
      await Promise.all(
        runningSessions.map(session =>
          invoke('pause_session', { sessionId: session.id })
        )
      );
      toast({
        title: 'Sessions paused',
        description: `Paused ${runningSessions.length} sessions`,
      });
    } catch (error) {
      toast({
        title: 'Failed to pause sessions',
        description: String(error),
        variant: 'destructive',
      });
    }
  };

  const resumeAll = async () => {
    const pausedSessions = sessions.filter(s => s.status === 'paused');

    if (pausedSessions.length === 0) {
      toast({
        title: 'No sessions to resume',
        description: 'No paused sessions found',
      });
      return;
    }

    try {
      await Promise.all(
        pausedSessions.map(session =>
          invoke('resume_session', { sessionId: session.id })
        )
      );
      toast({
        title: 'Sessions resumed',
        description: `Resumed ${pausedSessions.length} sessions`,
      });
    } catch (error) {
      toast({
        title: 'Failed to resume sessions',
        description: String(error),
        variant: 'destructive',
      });
    }
  };

  const terminateAll = async () => {
    const activeSessions = sessions.filter(s => s.status !== 'terminated');

    if (activeSessions.length === 0) {
      toast({
        title: 'No sessions to terminate',
        description: 'All sessions are already terminated',
      });
      return;
    }

    if (!confirm(`Are you sure you want to terminate ${activeSessions.length} sessions?`)) {
      return;
    }

    try {
      await Promise.all(
        activeSessions.map(session =>
          invoke('terminate_session', { sessionId: session.id })
        )
      );
      toast({
        title: 'Sessions terminated',
        description: `Terminated ${activeSessions.length} sessions`,
      });
    } catch (error) {
      toast({
        title: 'Failed to terminate sessions',
        description: String(error),
        variant: 'destructive',
      });
    }
  };

  return (
    <div className="flex items-center justify-between mt-4">
      <div className="flex items-center gap-2">
        <Badge variant="outline">
          Total: {sessionCounts.total}
        </Badge>
        {sessionCounts.running > 0 && (
          <Badge variant="default" className="bg-green-500">
            Running: {sessionCounts.running}
          </Badge>
        )}
        {sessionCounts.paused > 0 && (
          <Badge variant="secondary">
            Paused: {sessionCounts.paused}
          </Badge>
        )}
        {sessionCounts.error > 0 && (
          <Badge variant="destructive">
            Error: {sessionCounts.error}
          </Badge>
        )}
      </div>

      <div className="flex items-center gap-2">
        <Button
          variant="outline"
          size="sm"
          onClick={pauseAll}
          disabled={sessionCounts.running === 0 && sessionCounts.ready === 0}
        >
          <PauseCircle className="w-4 h-4 mr-2" />
          Pause All
        </Button>
        <Button
          variant="outline"
          size="sm"
          onClick={resumeAll}
          disabled={sessionCounts.paused === 0}
        >
          <PlayCircle className="w-4 h-4 mr-2" />
          Resume All
        </Button>
        <Button
          variant="outline"
          size="sm"
          onClick={terminateAll}
          disabled={sessionCounts.total === 0}
          className="text-destructive"
        >
          <StopCircle className="w-4 h-4 mr-2" />
          Terminate All
        </Button>
      </div>
    </div>
  );
}