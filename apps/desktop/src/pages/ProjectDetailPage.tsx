import { useEffect, useState } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import { useAppStore } from "../stores/appStore";
import { Button } from "../components/ui/Button";
import { FileTree } from "../components/FileTree";
import { TaskList } from "../components/TaskList";
import { TaskRunnerPanel, DecisionLogViewer } from "../components/TaskRunnerPanel";
import { FileChangesPanel } from "../components/FileChangesPanel";
import { TestRunner, TestHistory } from "../components/TestRunner";
import { DebugPanel, ErrorAnalyzer } from "../components/DebugPanel";
import { ScenarioPlanner, OrchestratorReportViewer } from "../components/Orchestrator";
import { Folder, ListTodo, Play, Bug, TestTube, GitBranch } from "lucide-react";

type RightPanelView = 'files' | 'tasks' | 'taskrunner' | 'tester' | 'debugger' | 'orchestrator';

export function ProjectDetailPage() {
  const { projectId } = useParams<{ projectId: string }>();
  const navigate = useNavigate();
  const {
    currentProject,
    fetchProject,
    fileTree,
    fetchFileTree,
    tasks,
    fetchTasks,
    projectRuns,
    fetchProjectRuns,
    createProjectRun,
    selectedFilePath,
    readFile,
  } = useAppStore();

  const [fileContent, setFileContent] = useState<string | null>(null);
  const [isRunning, setIsRunning] = useState(false);
  const [rightPanel, setRightPanel] = useState<RightPanelView>('files');

  useEffect(() => {
    if (projectId) {
      fetchProject(projectId);
      fetchProjectRuns(projectId);
    }
  }, [projectId, fetchProject, fetchProjectRuns]);

  useEffect(() => {
    if (currentProject) {
      fetchFileTree(currentProject.path);
    }
  }, [currentProject, fetchFileTree]);

  useEffect(() => {
    if (projectId) {
      fetchTasks(projectId);
    }
  }, [projectId, fetchTasks]);

  useEffect(() => {
    if (selectedFilePath) {
      readFile(selectedFilePath)
        .then(setFileContent)
        .catch(() => setFileContent("// Unable to read file"));
    } else {
      setFileContent(null);
    }
  }, [selectedFilePath, readFile]);

  const handleStartRun = async () => {
    if (!projectId) return;
    try {
      const run = await createProjectRun(projectId);
      setIsRunning(true);
      await invoke("start_run", { projectRunId: run.id });
      setIsRunning(false);
      await fetchProjectRuns(projectId);
    } catch (err) {
      setIsRunning(false);
      console.error(err);
    }
  };

  const handlePauseRun = () => {
    setIsRunning(false);
  };

  if (!currentProject) {
    return (
      <div className="flex-1 flex items-center justify-center">
        <p className="text-muted-foreground">Loading project...</p>
      </div>
    );
  }

  const currentRun = projectRuns[0];
  const activeTaskId = tasks[0]?.id;

  return (
    <div className="flex-1 flex flex-col overflow-hidden">
      {/* Project Header */}
      <div className="p-4 border-b">
        <div className="flex justify-between items-center">
          <div>
            <h1 className="text-xl font-bold">{currentProject.name}</h1>
            <p className="text-sm text-muted-foreground font-mono">{currentProject.path}</p>
          </div>
          <div className="flex gap-2">
            {isRunning ? (
              <Button variant="outline" onClick={handlePauseRun}>Pause</Button>
            ) : (
              <Button onClick={handleStartRun}>Start Run</Button>
            )}
            <Button variant="outline" onClick={() => navigate("/projects")}>Back</Button>
          </div>
        </div>

        {/* Run Status */}
        {currentRun && currentRun.status === "running" && (
          <div className="mt-3 space-y-1">
            <div className="flex justify-between text-sm">
              <span>Progress: {currentRun.progress_percent}%</span>
              <span>{currentRun.current_phase || "Initializing..."}</span>
            </div>
            <div className="h-2 bg-muted rounded overflow-hidden">
              <div
                className="h-full bg-primary transition-all"
                style={{ width: `${currentRun.progress_percent}%` }}
              />
            </div>
          </div>
        )}
      </div>

      {/* Main Content */}
      <div className="flex-1 flex overflow-hidden">
        {/* Left Panel: Navigation */}
        <div className="w-16 border-r bg-muted flex flex-col">
          <button
            className={`p-3 hover:bg-accent ${rightPanel === 'files' ? 'bg-accent' : ''}`}
            onClick={() => setRightPanel('files')}
            title="Files"
          >
            <Folder size={20} />
          </button>
          <button
            className={`p-3 hover:bg-accent ${rightPanel === 'tasks' ? 'bg-accent' : ''}`}
            onClick={() => setRightPanel('tasks')}
            title="Tasks"
          >
            <ListTodo size={20} />
          </button>
          <button
            className={`p-3 hover:bg-accent ${rightPanel === 'taskrunner' ? 'bg-accent' : ''}`}
            onClick={() => setRightPanel('taskrunner')}
            title="Task Runner"
          >
            <Play size={20} />
          </button>
          <button
            className={`p-3 hover:bg-accent ${rightPanel === 'tester' ? 'bg-accent' : ''}`}
            onClick={() => setRightPanel('tester')}
            title="Tester"
          >
            <TestTube size={20} />
          </button>
          <button
            className={`p-3 hover:bg-accent ${rightPanel === 'debugger' ? 'bg-accent' : ''}`}
            onClick={() => setRightPanel('debugger')}
            title="Debugger"
          >
            <Bug size={20} />
          </button>
          <button
            className={`p-3 hover:bg-accent ${rightPanel === 'orchestrator' ? 'bg-accent' : ''}`}
            onClick={() => setRightPanel('orchestrator')}
            title="Orchestrator"
          >
            <GitBranch size={20} />
          </button>
        </div>

        {/* Left Panel: Content */}
        <div className="w-80 border-r flex flex-col overflow-hidden">
          {rightPanel === 'files' && (
            <div className="flex-1 overflow-auto p-4">
              <h2 className="font-semibold mb-2">Files</h2>
              <FileTree nodes={fileTree} />
            </div>
          )}
          {rightPanel === 'tasks' && (
            <div className="flex-1 overflow-auto p-4">
              <h2 className="font-semibold mb-2">Tasks</h2>
              <TaskList tasks={tasks} />
            </div>
          )}
          {rightPanel === 'taskrunner' && (
            <div className="flex-1 overflow-auto p-4">
              <TaskRunnerPanel projectId={projectId || ''} />
              {currentRun && <DecisionLogViewer projectRunId={currentRun.id} />}
            </div>
          )}
          {rightPanel === 'tester' && (
            <div className="flex-1 overflow-auto p-4">
              <TestRunner projectId={projectId || ''} />
              <TestHistory projectId={projectId || ''} />
            </div>
          )}
          {rightPanel === 'debugger' && activeTaskId && (
            <div className="flex-1 overflow-auto p-4">
              <DebugPanel taskId={activeTaskId} />
              <ErrorAnalyzer />
            </div>
          )}
          {rightPanel === 'orchestrator' && (
            <div className="flex-1 overflow-auto p-4">
              <ScenarioPlanner projectId={projectId || ''} />
              {currentRun && <OrchestratorReportViewer projectRunId={currentRun.id} />}
            </div>
          )}
        </div>

        {/* Right Panel: Code Editor / Details */}
        <div className="flex-1 flex flex-col overflow-hidden">
          {rightPanel === 'files' && selectedFilePath ? (
            <>
              <div className="p-2 border-b bg-muted flex items-center justify-between">
                <span className="text-sm font-mono">{selectedFilePath}</span>
              </div>
              <div className="flex-1 overflow-auto">
                <pre className="p-4 text-sm font-mono whitespace-pre-wrap">{fileContent || "Loading..."}</pre>
              </div>
            </>
          ) : rightPanel === 'taskrunner' ? (
            <div className="flex-1 overflow-hidden">
              <FileChangesPanel projectId={projectId || ''} />
            </div>
          ) : rightPanel === 'tester' ? (
            <div className="flex-1 flex items-center justify-center text-muted-foreground">
              <div className="text-center">
                <TestTube size={48} className="mx-auto mb-4 opacity-50" />
                <p>Run tests to see results</p>
              </div>
            </div>
          ) : rightPanel === 'debugger' ? (
            <div className="flex-1 flex items-center justify-center text-muted-foreground">
              <div className="text-center">
                <Bug size={48} className="mx-auto mb-4 opacity-50" />
                <p>Start a debug session</p>
              </div>
            </div>
          ) : rightPanel === 'orchestrator' ? (
            <div className="flex-1 flex items-center justify-center text-muted-foreground">
              <div className="text-center">
                <GitBranch size={48} className="mx-auto mb-4 opacity-50" />
                <p>Create scenario plans</p>
              </div>
            </div>
          ) : (
            <div className="flex-1 flex items-center justify-center text-muted-foreground">
              <div className="text-center">
                <Folder size={48} className="mx-auto mb-4 opacity-50" />
                <p>Select a file to view its contents</p>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}