import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Button } from './ui/Button';
import { Card, CardHeader, CardTitle, CardContent } from './ui/Card';
import { Layers, Plus, ArrowRight, Loader2 } from 'lucide-react';

interface ScenarioPlanProps {
  projectId: string;
}

interface ScenarioPlan {
  id: string;
  name: string;
  description: string;
  complexity: string;
  estimated_tasks: number;
  estimated_duration_minutes: number;
  created_at: string;
}

interface TaskBreakdown {
  id: string;
  original_task_title: string;
  subtasks: Array<{
    task_id: string;
    title: string;
    task_type: string;
    description: string;
    dependencies: string[];
    complexity: string;
    risk_level: string;
    estimated_cost: number;
  }>;
  execution_order: string[];
}

export function ScenarioPlanner({ projectId }: ScenarioPlanProps) {
  const [plans, setPlans] = useState<ScenarioPlan[]>([]);
  const [loading, setLoading] = useState(false);
  const [creating, setCreating] = useState(false);

  const loadPlans = async () => {
    setLoading(true);
    try {
      const data = await invoke<ScenarioPlan[]>('get_scenario_plans', { projectId });
      setPlans(data);
    } catch (err) {
      console.error('Failed to load scenario plans:', err);
    } finally {
      setLoading(false);
    }
  };

  const createPlan = async () => {
    setCreating(true);
    try {
      const plan = await invoke<ScenarioPlan>('create_scenario_plan', {
        projectId,
        name: `Plan ${plans.length + 1}`,
        description: 'New scenario plan',
        complexity: 'medium',
      });
      setPlans([plan, ...plans]);
    } catch (err) {
      console.error('Failed to create plan:', err);
    } finally {
      setCreating(false);
    }
  };

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <CardTitle className="flex items-center gap-2">
            <Layers size={16} />
            Scenario Plans
          </CardTitle>
          <div className="flex gap-2">
            <Button variant="outline" size="sm" onClick={loadPlans}>
              <Loader2 size={14} className={`mr-1 ${loading ? 'animate-spin' : ''}`} />
              Refresh
            </Button>
            <Button size="sm" onClick={createPlan} disabled={creating}>
              <Plus size={14} className="mr-1" />
              New Plan
            </Button>
          </div>
        </div>
      </CardHeader>
      <CardContent>
        {loading ? (
          <div className="flex items-center justify-center py-4">
            <Loader2 size={16} className="animate-spin" />
          </div>
        ) : plans.length === 0 ? (
          <p className="text-sm text-muted-foreground text-center py-4">
            No scenario plans yet. Create one to get started.
          </p>
        ) : (
          <div className="space-y-3">
            {plans.map((plan) => (
              <div key={plan.id} className="p-3 bg-muted rounded">
                <div className="flex items-center justify-between">
                  <span className="font-medium">{plan.name}</span>
                  <span className={`text-xs px-2 py-1 rounded ${
                    plan.complexity === 'complex'
                      ? 'bg-red-100 text-red-700'
                      : plan.complexity === 'medium'
                      ? 'bg-yellow-100 text-yellow-700'
                      : 'bg-green-100 text-green-700'
                  }`}>
                    {plan.complexity}
                  </span>
                </div>
                <p className="text-sm text-muted-foreground mt-1">
                  {plan.description || 'No description'}
                </p>
                <div className="flex items-center gap-4 mt-2 text-xs text-muted-foreground">
                  <span>~{plan.estimated_tasks} tasks</span>
                  <span>~{plan.estimated_duration_minutes} min</span>
                </div>
              </div>
            ))}
          </div>
        )}
      </CardContent>
    </Card>
  );
}

export function TaskBreakdownView({ scenarioPlanId }: { scenarioPlanId: string }) {
  const [breakdowns, setBreakdowns] = useState<TaskBreakdown[]>([]);
  const [creating, setCreating] = useState(false);

  const createBreakdown = async () => {
    setCreating(true);
    try {
      const breakdown = await invoke<TaskBreakdown>('breakdown_task', {
        scenarioPlanId,
        originalTaskTitle: 'New Task',
        taskType: 'frontend_coding',
        complexity: 'medium',
      });
      setBreakdowns(prev => [...prev, breakdown]);
    } catch (err) {
      console.error('Failed to create breakdown:', err);
    } finally {
      setCreating(false);
    }
  };

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <CardTitle>Task Breakdown</CardTitle>
          <Button size="sm" onClick={createBreakdown} disabled={creating}>
            <Plus size={14} className="mr-1" />
            Breakdown Task
          </Button>
        </div>
      </CardHeader>
      <CardContent>
        {breakdowns.length === 0 ? (
          <p className="text-sm text-muted-foreground text-center py-4">
            No breakdowns yet. Create one to decompose tasks.
          </p>
        ) : (
          <div className="space-y-4">
            {breakdowns.map((bd) => (
              <div key={bd.id} className="space-y-2">
                <h4 className="font-medium">{bd.original_task_title}</h4>
                <div className="pl-4 space-y-2 border-l-2 border-muted">
                  {bd.subtasks.map((subtask, index) => (
                    <div key={subtask.task_id} className="flex items-start gap-2">
                      <div className="flex flex-col items-center">
                        <span className="text-xs bg-muted px-1.5 py-0.5 rounded">
                          {index + 1}
                        </span>
                        {index < bd.subtasks.length - 1 && (
                          <ArrowRight size={12} className="mt-1 text-muted-foreground" />
                        )}
                      </div>
                      <div className="flex-1 p-2 bg-muted rounded">
                        <div className="flex items-center justify-between">
                          <span className="text-sm font-medium">{subtask.title}</span>
                          <span className="text-xs text-muted-foreground">
                            {subtask.task_type}
                          </span>
                        </div>
                        <p className="text-xs text-muted-foreground mt-1">
                          {subtask.description}
                        </p>
                        {subtask.dependencies.length > 0 && (
                          <p className="text-xs text-muted-foreground mt-1">
                            Dependencies: {subtask.dependencies.join(', ')}
                          </p>
                        )}
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            ))}
          </div>
        )}
      </CardContent>
    </Card>
  );
}

// Orchestrator report viewer
export function OrchestratorReportViewer({ projectRunId }: { projectRunId: string }) {
  const [reports, setReports] = useState<Array<{
    id: string;
    report_type: string;
    title: string;
    summary: string;
    progress_percent: number;
    created_at: string;
  }>>([]);
  const [loading, setLoading] = useState(false);

  const loadReports = async () => {
    setLoading(true);
    try {
      const data = await invoke<Array<{
        id: string;
        report_type: string;
        title: string;
        summary: string;
        progress_percent: number;
        created_at: string;
      }>>('get_orchestrator_reports', { projectRunId });
      setReports(data);
    } catch (err) {
      console.error('Failed to load reports:', err);
    } finally {
      setLoading(false);
    }
  };

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <CardTitle>Orchestrator Reports</CardTitle>
          <Button variant="outline" size="sm" onClick={loadReports}>
            <Loader2 size={14} className={`mr-1 ${loading ? 'animate-spin' : ''}`} />
            Refresh
          </Button>
        </div>
      </CardHeader>
      <CardContent>
        {loading ? (
          <div className="flex items-center justify-center py-4">
            <Loader2 size={16} className="animate-spin" />
          </div>
        ) : reports.length === 0 ? (
          <p className="text-sm text-muted-foreground text-center py-4">
            No reports yet
          </p>
        ) : (
          <div className="space-y-3">
            {reports.map((report) => (
              <div key={report.id} className="p-3 bg-muted rounded">
                <div className="flex items-center justify-between">
                  <span className="font-medium">{report.title}</span>
                  <span className="text-xs bg-primary/10 text-primary px-2 py-1 rounded">
                    {report.report_type}
                  </span>
                </div>
                <p className="text-sm mt-1">{report.summary}</p>
                <div className="flex items-center justify-between mt-2">
                  <div className="flex items-center gap-2">
                    <div className="w-24 h-2 bg-muted-foreground/20 rounded overflow-hidden">
                      <div
                        className="h-full bg-primary"
                        style={{ width: `${report.progress_percent}%` }}
                      />
                    </div>
                    <span className="text-xs">{report.progress_percent}%</span>
                  </div>
                  <span className="text-xs text-muted-foreground">
                    {new Date(report.created_at).toLocaleString()}
                  </span>
                </div>
              </div>
            ))}
          </div>
        )}
      </CardContent>
    </Card>
  );
}