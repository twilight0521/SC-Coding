import { useEffect, useState } from "react";
import { useAppStore } from "../stores/appStore";
import { Card, CardHeader, CardTitle, CardContent } from "../components/ui/Card";
import { Button } from "../components/ui/Button";
import { Input, Label } from "../components/ui/Input";

export function AgentsPage() {
  const { agents, fetchAgents, createAgent, deleteAgent, createDefaultAgents } = useAppStore();
  const [showForm, setShowForm] = useState(false);
  const [formData, setFormData] = useState({
    name: "",
    role: "orchestrator",
    description: "",
    system_prompt: "",
  });

  useEffect(() => {
    fetchAgents();
  }, [fetchAgents]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      await createAgent({
        name: formData.name,
        role: formData.role,
        description: formData.description || undefined,
        system_prompt: formData.system_prompt,
      });
      setShowForm(false);
      setFormData({ name: "", role: "orchestrator", description: "", system_prompt: "" });
    } catch (err) {
      console.error(err);
    }
  };

  const handleCreateDefaults = async () => {
    try {
      await createDefaultAgents();
    } catch (err) {
      console.error(err);
    }
  };

  return (
    <div className="flex-1 p-6 overflow-auto">
      <div className="flex justify-between items-center mb-6">
        <h1 className="text-2xl font-bold">Agents</h1>
        <div className="flex gap-2">
          {agents.length === 0 && (
            <Button variant="outline" onClick={handleCreateDefaults}>
              Create Default Agents
            </Button>
          )}
          <Button onClick={() => setShowForm(!showForm)}>
            {showForm ? "Cancel" : "Create Agent"}
          </Button>
        </div>
      </div>

      {/* Create Agent Form */}
      {showForm && (
        <Card className="mb-6">
          <CardContent className="pt-6">
            <form onSubmit={handleSubmit} className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <Label>Name</Label>
                  <Input value={formData.name} onChange={(e) => setFormData({ ...formData, name: e.target.value })} required />
                </div>
                <div>
                  <Label>Role</Label>
                  <select
                    className="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm"
                    value={formData.role}
                    onChange={(e) => setFormData({ ...formData, role: e.target.value })}
                  >
                    <option value="orchestrator">Orchestrator</option>
                    <option value="product_manager">Product Manager</option>
                    <option value="architect">Architect</option>
                    <option value="frontend_engineer">Frontend Engineer</option>
                    <option value="backend_engineer">Backend Engineer</option>
                    <option value="fullstack_engineer">Fullstack Engineer</option>
                    <option value="test_engineer">Test Engineer</option>
                    <option value="debug_engineer">Debug Engineer</option>
                    <option value="security_reviewer">Security Reviewer</option>
                    <option value="code_reviewer">Code Reviewer</option>
                    <option value="integration_engineer">Integration Engineer</option>
                    <option value="document_writer">Document Writer</option>
                    <option value="researcher">Researcher</option>
                    <option value="cost_controller">Cost Controller</option>
                  </select>
                </div>
              </div>
              <div>
                <Label>Description</Label>
                <Input value={formData.description} onChange={(e) => setFormData({ ...formData, description: e.target.value })} />
              </div>
              <div>
                <Label>System Prompt</Label>
                <textarea
                  className="flex min-h-32 w-full rounded-md border border-input bg-transparent px-3 py-2 text-sm"
                  value={formData.system_prompt}
                  onChange={(e) => setFormData({ ...formData, system_prompt: e.target.value })}
                  placeholder="Enter the agent's system prompt..."
                  required
                />
              </div>
              <Button type="submit">Create Agent</Button>
            </form>
          </CardContent>
        </Card>
      )}

      {/* Agents Grid */}
      {agents.length === 0 ? (
        <div className="text-center py-12">
          <p className="text-muted-foreground mb-4">No agents configured</p>
          <Button onClick={handleCreateDefaults}>Create Default Agents</Button>
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {agents.map((agent) => (
            <Card key={agent.id}>
              <CardHeader>
                <CardTitle>{agent.name}</CardTitle>
                <span className="text-xs text-muted-foreground">{agent.role}</span>
              </CardHeader>
              <CardContent>
                <p className="text-sm text-muted-foreground mb-2">{agent.description || "No description"}</p>
                <p className="text-xs text-muted-foreground mb-4 truncate">
                  {agent.system_prompt.substring(0, 100)}...
                </p>
                <div className="flex justify-between items-center">
                  <span className={`px-2 py-1 text-xs rounded ${agent.is_enabled ? "bg-green-100" : "bg-gray-100"}`}>
                    {agent.is_enabled ? "Active" : "Inactive"}
                  </span>
                  <Button variant="outline" size="sm" onClick={() => deleteAgent(agent.id)}>
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