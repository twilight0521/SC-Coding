import { useAppStore, Task } from "../stores/appStore";
import { CheckCircle, Circle, Clock, AlertCircle } from "lucide-react";

interface TaskListProps {
  tasks: Task[];
}

function TaskItem({ task }: { task: Task }) {
  const { updateTaskStatus } = useAppStore();

  const statusIcon = {
    pending: <Circle size={14} className="text-gray-400" />,
    in_progress: <Clock size={14} className="text-blue-400" />,
    completed: <CheckCircle size={14} className="text-green-500" />,
    failed: <AlertCircle size={14} className="text-red-500" />,
  };

  return (
    <div className="flex items-start gap-2 p-2 hover:bg-accent rounded cursor-pointer"
         onClick={() => {
           const nextStatus = task.status === "pending" ? "in_progress" : task.status;
           updateTaskStatus(task.id, nextStatus);
         }}>
      <span className="mt-0.5">{statusIcon[task.status as keyof typeof statusIcon] || statusIcon.pending}</span>
      <div className="flex-1 min-w-0">
        <p className="text-sm font-medium truncate">{task.title}</p>
        <p className="text-xs text-muted-foreground">{task.task_type}</p>
      </div>
    </div>
  );
}

export function TaskList({ tasks }: TaskListProps) {
  if (tasks.length === 0) {
    return <p className="text-sm text-muted-foreground p-2">No tasks yet</p>;
  }

  return (
    <div className="space-y-1">
      {tasks.map((task) => (
        <TaskItem key={task.id} task={task} />
      ))}
    </div>
  );
}