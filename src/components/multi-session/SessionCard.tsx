import { Card } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Play, Pause, Square, GitBranch, Clock, FileCode } from 'lucide-react';
import { SessionInfo } from '@/types/multi-session';
import { cn } from '@/lib/utils';

interface SessionCardProps {
  session: SessionInfo;
  isActive: boolean;
  onClick: () => void;
}

export default function SessionCard({ session, isActive, onClick }: SessionCardProps) {
  const getStatusColor = (status: string) => {
    switch (status) {
      case 'running':
        return 'bg-green-500';
      case 'ready':
        return 'bg-blue-500';
      case 'loading':
        return 'bg-yellow-500';
      case 'paused':
        return 'bg-gray-500';
      case 'error':
        return 'bg-red-500';
      case 'completed':
        return 'bg-purple-500';
      default:
        return 'bg-gray-400';
    }
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'running':
      case 'loading':
        return <Play className="w-3 h-3" />;
      case 'paused':
        return <Pause className="w-3 h-3" />;
      case 'terminated':
      case 'error':
        return <Square className="w-3 h-3" />;
      default:
        return null;
    }
  };

  const formatTime = (dateString: string) => {
    const date = new Date(dateString);
    const now = new Date();
    const diff = now.getTime() - date.getTime();
    const minutes = Math.floor(diff / 60000);
    const hours = Math.floor(minutes / 60);
    const days = Math.floor(hours / 24);

    if (days > 0) return `${days}d ago`;
    if (hours > 0) return `${hours}h ago`;
    if (minutes > 0) return `${minutes}m ago`;
    return 'Just now';
  };

  return (
    <Card
      className={cn(
        'p-4 cursor-pointer transition-all hover:shadow-md',
        isActive && 'ring-2 ring-primary'
      )}
      onClick={onClick}
    >
      <div className="space-y-2">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <div className={cn('w-2 h-2 rounded-full', getStatusColor(session.status))} />
            <span className="font-medium text-sm">
              Session {session.id.slice(0, 8)}
            </span>
          </div>
          <Badge variant="outline" className="text-xs">
            {getStatusIcon(session.status)}
            {session.status}
          </Badge>
        </div>

        <div className="flex items-center gap-4 text-xs text-muted-foreground">
          <div className="flex items-center gap-1">
            <GitBranch className="w-3 h-3" />
            <span className="truncate max-w-[120px]">{session.branch_name}</span>
          </div>
          <div className="flex items-center gap-1">
            <Clock className="w-3 h-3" />
            <span>{formatTime(session.created_at)}</span>
          </div>
        </div>

        {session.diff_stats && (
          <div className="flex items-center gap-2 text-xs">
            <FileCode className="w-3 h-3 text-muted-foreground" />
            <span className="text-green-600">+{session.diff_stats.insertions}</span>
            <span className="text-red-600">-{session.diff_stats.deletions}</span>
            <span className="text-muted-foreground">
              ({session.diff_stats.files_changed} files)
            </span>
          </div>
        )}

        {session.output_preview && (
          <div className="text-xs text-muted-foreground truncate">
            {session.output_preview.split('\n').slice(-1)[0]}
          </div>
        )}

        {session.auto_yes && (
          <Badge variant="secondary" className="text-xs">
            Auto-Yes
          </Badge>
        )}
      </div>
    </Card>
  );
}