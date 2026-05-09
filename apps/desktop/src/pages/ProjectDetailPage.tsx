import { useEffect, useState } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { useAppStore } from "../stores/appStore";
import { Button } from "../components/ui/Button";
import { FileTree } from "../components/FileTree";
import { TaskList } from "../components/TaskList";

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
      await createProjectRun(projectId);
      setIsRunning(true);
    } catch (err) {
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

  return (
    <div className="flex-1 flex flex-col overflow-hidden">
      {/* Project Header */}
      <div className="p-6 border-b">
        <div className="flex justify-between items-center">
          <div>
            <h1 className="text-2xl font-bold">{currentProject.name}</h1>
            <p className="text-muted-foreground text-sm">{currentProject.path}</p>
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
        {projectRuns.length > 0 && (
          <div className="mt-4">
            {projectRuns[0].status === "running" && (
              <div className="space-y-2">
                <div className="flex justify-between text-sm">
                  <span>Progress: {projectRuns[0].progress_percent}%</span>
                  <span>{projectRuns[0].current_phase || "Initializing..."}</span>
                </div>
                <div className="h-2 bg-muted rounded overflow-hidden">
                  <div
                    className="h-full bg-primary transition-all"
                    style={{ width: `${projectRuns[0].progress_percent}%` }}
                  />
                </div>
              </div>
            )}
          </div>
        )}
      </div>

      {/* Main Content */}
      <div className="flex-1 flex overflow-hidden">
        {/* Left Panel: File Tree + Tasks */}
        <div className="w-80 border-r flex flex-col overflow-hidden">
          {/* File Tree */}
          <div className="flex-1 overflow-auto p-4 border-b">
            <h2 className="font-semibold mb-2">Files</h2>
            <FileTree nodes={fileTree} />
          </div>

          {/* Tasks */}
          <div className="flex-1 overflow-auto p-4">
            <h2 className="font-semibold mb-2">Tasks</h2>
            <TaskList tasks={tasks} />
          </div>
        </div>

        {/* Right Panel: Code Editor / Logs */}
        <div className="flex-1 flex flex-col overflow-hidden">
          {selectedFilePath ? (
            <>
              <div className="p-2 border-b bg-muted">
                <span className="text-sm font-mono">{selectedFilePath}</span>
              </div>
              <div className="flex-1 overflow-auto p-4">
                <pre className="text-sm font-mono whitespace-pre-wrap">{fileContent || "Loading..."}</pre>
              </div>
            </>
          ) : (
            <div className="flex-1 flex items-center justify-center text-muted-foreground">
              <p>Select a file to view its contents</p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}