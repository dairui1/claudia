import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { Plus, RefreshCw, ArrowLeft, FolderOpen } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Toast, ToastContainer } from '@/components/ui/toast';
import { SelectComponent } from '@/components/ui/select';
import SessionCard from './SessionCard';
import SessionPreview from './SessionPreview';
import SessionDiffView from './SessionDiffView';
import GlobalSessionControls from './GlobalSessionControls';
import { SessionInfo, SessionEvent, SessionConfig } from '@/types/multi-session';
import { api, type Project } from '@/lib/api';

interface MultiSessionDashboardProps {
  onBack?: () => void;
}

export default function MultiSessionDashboard({ onBack }: MultiSessionDashboardProps) {
  const [sessions, setSessions] = useState<Map<string, SessionInfo>>(new Map());
  const [activeSessionId, setActiveSessionId] = useState<string | null>(null);
  const [isCreating, setIsCreating] = useState(false);
  const [projects, setProjects] = useState<Project[]>([]);
  const [selectedProject, setSelectedProject] = useState<string>('');
  const [loadingProjects, setLoadingProjects] = useState(false);
  const [config] = useState<SessionConfig>({
    auto_yes: false,
    max_output_buffer: 10000,
    environment_vars: [],
    working_directory: null,
    branch_prefix: 'claudia-session',
    claude_args: [],
  });
  const [toast, setToast] = useState<{ message: string; type: "success" | "error" | "info" } | null>(null);

  useEffect(() => {
    loadSessions();
    loadProjects();
    
    const unlistenPromise = listen<SessionEvent>('session-event', (event) => {
      handleSessionEvent(event.payload);
    });

    return () => {
      unlistenPromise.then(unlisten => unlisten());
    };
  }, []);

  const loadProjects = async () => {
    try {
      setLoadingProjects(true);
      const projectList = await api.listProjects();
      setProjects(projectList);
    } catch (error) {
      console.error('Failed to load projects:', error);
      setToast({
        message: 'Failed to load projects',
        type: 'error',
      });
    } finally {
      setLoadingProjects(false);
    }
  };

  const loadSessions = async () => {
    try {
      const sessionList = await invoke<SessionInfo[]>('list_active_sessions');
      const newSessions = new Map();
      sessionList.forEach(session => {
        newSessions.set(session.id, session);
      });
      setSessions(newSessions);
      
      if (sessionList.length > 0 && !activeSessionId) {
        setActiveSessionId(sessionList[0].id);
      }
    } catch (error) {
      console.error('Failed to load sessions:', error);
    }
  };

  const handleSessionEvent = (event: SessionEvent) => {
    switch (event.type) {
      case 'StatusChanged':
        updateSessionStatus(event.data.session_id, event.data.status);
        break;
      case 'OutputAppended':
        updateSessionOutput(event.data.session_id, event.data.output);
        break;
      case 'DiffUpdated':
        updateSessionDiff(event.data.session_id, event.data.stats);
        break;
      case 'SessionCreated':
        loadSessions();
        setActiveSessionId(event.data.session_id);
        break;
      case 'SessionTerminated':
        removeSession(event.data.session_id);
        break;
      case 'Error':
        setToast({
          message: event.data.error,
          type: 'error',
        });
        break;
    }
  };

  const updateSessionStatus = (sessionId: string, status: any) => {
    setSessions(prev => {
      const newSessions = new Map(prev);
      const session = newSessions.get(sessionId);
      if (session) {
        session.status = status;
        newSessions.set(sessionId, { ...session });
      }
      return newSessions;
    });
  };

  const updateSessionOutput = (sessionId: string, output: string) => {
    setSessions(prev => {
      const newSessions = new Map(prev);
      const session = newSessions.get(sessionId);
      if (session) {
        session.output_preview = session.output_preview + '\n' + output;
        const lines = session.output_preview.split('\n');
        if (lines.length > 100) {
          session.output_preview = lines.slice(-100).join('\n');
        }
        newSessions.set(sessionId, { ...session });
      }
      return newSessions;
    });
  };

  const updateSessionDiff = (sessionId: string, stats: any) => {
    setSessions(prev => {
      const newSessions = new Map(prev);
      const session = newSessions.get(sessionId);
      if (session) {
        session.diff_stats = stats;
        newSessions.set(sessionId, { ...session });
      }
      return newSessions;
    });
  };

  const removeSession = (sessionId: string) => {
    setSessions(prev => {
      const newSessions = new Map(prev);
      newSessions.delete(sessionId);
      if (activeSessionId === sessionId) {
        const remaining = Array.from(newSessions.keys());
        setActiveSessionId(remaining.length > 0 ? remaining[0] : null);
      }
      return newSessions;
    });
  };

  const createSession = async () => {
    if (!selectedProject) {
      setToast({
        message: 'Please select a project to create a session',
        type: 'error',
      });
      return;
    }

    setIsCreating(true);
    try {
      const sessionId = await invoke<string>('create_multi_session', {
        projectId: selectedProject,
        config,
      });
      setToast({
        message: `Session ${sessionId.slice(0, 8)} created successfully`,
        type: 'success',
      });
    } catch (error) {
      setToast({
        message: String(error),
        type: 'error',
      });
    } finally {
      setIsCreating(false);
    }
  };

  const activeSession = activeSessionId ? sessions.get(activeSessionId) : null;

  return (
    <div className="flex flex-col h-full">
      <div className="border-b p-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-4">
            {onBack && (
              <Button
                variant="ghost"
                size="sm"
                onClick={onBack}
              >
                <ArrowLeft className="w-4 h-4 mr-2" />
                Back
              </Button>
            )}
            <h2 className="text-2xl font-bold">Multi-Session Manager</h2>
          </div>
          <div className="flex items-center gap-2">
            <Button
              onClick={createSession}
              disabled={isCreating || !selectedProject}
              size="sm"
            >
              <Plus className="w-4 h-4 mr-2" />
              New Session
            </Button>
            <Button
              onClick={loadSessions}
              variant="outline"
              size="sm"
            >
              <RefreshCw className="w-4 h-4" />
            </Button>
          </div>
        </div>

        {/* Project Selection */}
        <div className="mt-4 flex items-center gap-4">
          <div className="flex items-center gap-2">
            <FolderOpen className="w-4 h-4 text-muted-foreground" />
            <Label className="text-sm font-medium">Project:</Label>
          </div>
          <div className="min-w-[300px]">
            <SelectComponent
              value={selectedProject}
              onValueChange={setSelectedProject}
              options={projects.map(project => ({
                value: project.id,
                label: project.path,
              }))}
              placeholder={loadingProjects ? "Loading projects..." : "Select a project"}
              disabled={loadingProjects || isCreating}
            />
          </div>
          <Button
            onClick={loadProjects}
            variant="outline"
            size="sm"
            disabled={loadingProjects}
          >
            <RefreshCw className={`w-4 h-4 ${loadingProjects ? 'animate-spin' : ''}`} />
          </Button>
        </div>

        <GlobalSessionControls sessions={Array.from(sessions.values())} />
      </div>

      <div className="flex flex-1 overflow-hidden">
        <div className="w-1/3 border-r">
          <ScrollArea className="h-full">
            <div className="p-4 space-y-2">
              <div className="text-sm text-muted-foreground mb-2">
                Total: {sessions.size}
              </div>
              {Array.from(sessions.values()).map(session => (
                <SessionCard
                  key={session.id}
                  session={session}
                  isActive={session.id === activeSessionId}
                  onClick={() => setActiveSessionId(session.id)}
                />
              ))}
              {sessions.size === 0 && (
                <div className="text-center text-muted-foreground py-8">
                  No active sessions
                </div>
              )}
            </div>
          </ScrollArea>
        </div>

        <div className="flex-1">
          {activeSession ? (
            <Tabs value="preview" onValueChange={() => {}} className="h-full">
              <TabsList className="w-full justify-start rounded-none border-b">
                <TabsTrigger value="preview">Output</TabsTrigger>
                <TabsTrigger value="diff">Changes</TabsTrigger>
                <TabsTrigger value="config">Config</TabsTrigger>
              </TabsList>
              
              <TabsContent value="preview" className="flex-1 p-0">
                <SessionPreview session={activeSession} />
              </TabsContent>
              
              <TabsContent value="diff" className="flex-1 p-0">
                <SessionDiffView sessionId={activeSession.id} />
              </TabsContent>
              
              <TabsContent value="config" className="p-4">
                <div className="space-y-4 max-w-2xl">
                  <div>
                    <Label htmlFor="auto-yes">Auto-Yes Mode</Label>
                    <Input
                      id="auto-yes"
                      type="checkbox"
                      checked={activeSession.auto_yes}
                      onChange={async (e) => {
                        try {
                          await invoke('update_session_config', {
                            sessionId: activeSession.id,
                            config: { ...config, auto_yes: e.target.checked },
                          });
                          loadSessions();
                        } catch (error) {
                          setToast({
                            message: String(error),
                            type: 'error',
                          });
                        }
                      }}
                    />
                  </div>
                </div>
              </TabsContent>
            </Tabs>
          ) : (
            <div className="flex items-center justify-center h-full text-muted-foreground">
              Select a session to view details
            </div>
          )}
        </div>
      </div>
      
      {/* Toast Container */}
      <ToastContainer>
        {toast && (
          <Toast
            message={toast.message}
            type={toast.type}
            onDismiss={() => setToast(null)}
          />
        )}
      </ToastContainer>
    </div>
  );
}