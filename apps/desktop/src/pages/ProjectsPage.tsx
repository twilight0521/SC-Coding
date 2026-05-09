import { useEffect, useState } from "react";
import { useAppStore } from "../stores/appStore";
import { Card, CardHeader, CardTitle, CardContent } from "../components/ui/Card";
import { Button } from "../components/ui/Button";
import { Input, Label } from "../components/ui/Input";
import { useNavigate } from "react-router-dom";

export function ProjectsPage() {
  const { projects, fetchProjects, createProject, deleteProject } = useAppStore();
  const [showForm, setShowForm] = useState(false);
  const [formData, setFormData] = useState({
    name: "",
    path: "",
    project_type: "",
    tech_stack: "",
  });
  const navigate = useNavigate();

  useEffect(() => {
    fetchProjects();
  }, [fetchProjects]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      const project = await createProject({
        name: formData.name,
        path: formData.path,
        project_type: formData.project_type || undefined,
        tech_stack: formData.tech_stack || undefined,
      });
      setShowForm(false);
      setFormData({ name: "", path: "", project_type: "", tech_stack: "" });
      navigate(`/project/${project.id}`);
    } catch (err) {
      console.error(err);
    }
  };

  return (
    <div className="flex-1 p-6 overflow-auto">
      <div className="flex justify-between items-center mb-6">
        <h1 className="text-2xl font-bold">Projects</h1>
        <Button onClick={() => setShowForm(!showForm)}>
          {showForm ? "Cancel" : "New Project"}
        </Button>
      </div>

      {/* Create Project Form */}
      {showForm && (
        <Card className="mb-6">
          <CardContent className="pt-6">
            <form onSubmit={handleSubmit} className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <Label>Project Name</Label>
                  <Input
                    value={formData.name}
                    onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                    placeholder="My Awesome Project"
                    required
                  />
                </div>
                <div>
                  <Label>Project Type</Label>
                  <select
                    className="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm"
                    value={formData.project_type}
                    onChange={(e) => setFormData({ ...formData, project_type: e.target.value })}
                  >
                    <option value="">Select type...</option>
                    <option value="web">Web Application</option>
                    <option value="mobile">Mobile App</option>
                    <option value="desktop">Desktop App</option>
                    <option value="api">API / Backend</option>
                    <option value="library">Library / Package</option>
                    <option value="other">Other</option>
                  </select>
                </div>
              </div>
              <div>
                <Label>Project Path</Label>
                <Input
                  value={formData.path}
                  onChange={(e) => setFormData({ ...formData, path: e.target.value })}
                  placeholder="/path/to/your/project"
                  required
                />
                <p className="text-xs text-muted-foreground mt-1">
                  Enter the absolute path to your project folder
                </p>
              </div>
              <div>
                <Label>Tech Stack (Optional)</Label>
                <Input
                  value={formData.tech_stack}
                  onChange={(e) => setFormData({ ...formData, tech_stack: e.target.value })}
                  placeholder="React, Node.js, PostgreSQL, etc."
                />
              </div>
              <Button type="submit">Create Project</Button>
            </form>
          </CardContent>
        </Card>
      )}

      {/* Projects Grid */}
      {projects.length === 0 ? (
        <div className="text-center py-12">
          <p className="text-muted-foreground mb-4">No projects yet</p>
          <Button onClick={() => setShowForm(true)}>Create Your First Project</Button>
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {projects.map((project) => (
            <Card key={project.id} className="hover:border-primary cursor-pointer transition-colors"
                  onClick={() => navigate(`/project/${project.id}`)}>
              <CardHeader>
                <CardTitle>{project.name}</CardTitle>
                {project.project_type && (
                  <span className="text-xs text-muted-foreground">{project.project_type}</span>
                )}
              </CardHeader>
              <CardContent>
                <p className="text-sm text-muted-foreground mb-2 truncate">{project.path}</p>
                {project.tech_stack && (
                  <p className="text-xs text-muted-foreground mb-4">{project.tech_stack}</p>
                )}
                <div className="flex justify-between items-center">
                  <span className="text-xs text-muted-foreground">
                    Created: {new Date(project.created_at).toLocaleDateString()}
                  </span>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={(e) => {
                      e.stopPropagation();
                      if (confirm("Delete this project?")) {
                        deleteProject(project.id);
                      }
                    }}
                  >
                    Delete
                  </Button>
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      )}
    </div>
  );
}