import { useEffect, useState } from "react";
import { useAppStore } from "../stores/appStore";
import { Card, CardHeader, CardTitle, CardContent } from "../components/ui/Card";
import { Button } from "../components/ui/Button";
import { Input, Label } from "../components/ui/Input";

export function ProvidersPage() {
  const { providers, presets, fetchProviders, fetchPresets, createProvider, deleteProvider, testConnection } = useAppStore();
  const [showForm, setShowForm] = useState(false);
  const [formData, setFormData] = useState({
    name: "",
    provider_type: "openai_compatible",
    base_url: "",
    api_key: "",
    default_model_id: "",
    display_model_name: "",
  });
  const [testResult, setTestResult] = useState<{ success: boolean; latency_ms?: number; error?: string } | null>(null);

  useEffect(() => {
    fetchProviders();
    fetchPresets();
  }, [fetchProviders, fetchPresets]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      await createProvider({
        name: formData.name,
        provider_type: formData.provider_type,
        base_url: formData.base_url,
        api_key: formData.api_key || undefined,
        default_model_id: formData.default_model_id,
        display_model_name: formData.display_model_name || undefined,
      });
      setShowForm(false);
      setFormData({ name: "", provider_type: "openai_compatible", base_url: "", api_key: "", default_model_id: "", display_model_name: "" });
    } catch (err) {
      console.error(err);
    }
  };

  const handleTest = async () => {
    const result = await testConnection(formData.base_url, formData.api_key || null, formData.default_model_id);
    setTestResult(result);
  };

  const applyPreset = (preset: typeof presets[0]) => {
    setFormData({
      name: preset.name,
      provider_type: preset.provider_type,
      base_url: preset.base_url,
      api_key: "",
      default_model_id: preset.default_model,
      display_model_name: preset.default_model,
    });
    setShowForm(true);
  };

  return (
    <div className="flex-1 p-6 overflow-auto">
      <div className="flex justify-between items-center mb-6">
        <h1 className="text-2xl font-bold">Model Providers</h1>
        <Button onClick={() => setShowForm(!showForm)}>
          {showForm ? "Cancel" : "Add Provider"}
        </Button>
      </div>

      {/* Add Provider Form */}
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
                  <Label>Provider Type</Label>
                  <select
                    className="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm"
                    value={formData.provider_type}
                    onChange={(e) => setFormData({ ...formData, provider_type: e.target.value })}
                  >
                    <option value="openai_compatible">OpenAI Compatible</option>
                    <option value="minimax">Minimax</option>
                    <option value="deepseek">DeepSeek</option>
                    <option value="ollama">Ollama</option>
                    <option value="lmstudio">LM Studio</option>
                  </select>
                </div>
              </div>
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <Label>Base URL</Label>
                  <Input value={formData.base_url} onChange={(e) => setFormData({ ...formData, base_url: e.target.value })} required />
                </div>
                <div>
                  <Label>API Key</Label>
                  <Input type="password" value={formData.api_key} onChange={(e) => setFormData({ ...formData, api_key: e.target.value })} />
                </div>
              </div>
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <Label>Default Model ID</Label>
                  <Input value={formData.default_model_id} onChange={(e) => setFormData({ ...formData, default_model_id: e.target.value })} required />
                </div>
                <div>
                  <Label>Display Name</Label>
                  <Input value={formData.display_model_name} onChange={(e) => setFormData({ ...formData, display_model_name: e.target.value })} />
                </div>
              </div>
              <div className="flex gap-2">
                <Button type="submit">Create Provider</Button>
                <Button type="button" variant="outline" onClick={handleTest}>Test Connection</Button>
              </div>
              {testResult && (
                <div className={`p-3 rounded ${testResult.success ? "bg-green-100" : "bg-red-100"}`}>
                  {testResult.success ? (
                    <span className="text-green-800">Connection successful! Latency: {testResult.latency_ms}ms</span>
                  ) : (
                    <span className="text-red-800">Connection failed: {testResult.error}</span>
                  )}
                </div>
              )}
            </form>
          </CardContent>
        </Card>
      )}

      {/* Presets */}
      <section className="mb-8">
        <h2 className="text-lg font-semibold mb-4">Quick Presets</h2>
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
          {presets.map((preset) => (
            <Card key={preset.id} className="cursor-pointer hover:border-primary" onClick={() => applyPreset(preset)}>
              <CardHeader>
                <CardTitle>{preset.name}</CardTitle>
              </CardHeader>
              <CardContent>
                <p className="text-sm text-muted-foreground">{preset.default_model}</p>
                <p className="text-xs text-muted-foreground mt-1 truncate">{preset.base_url}</p>
              </CardContent>
            </Card>
          ))}
        </div>
      </section>

      {/* Custom Providers */}
      <section>
        <h2 className="text-lg font-semibold mb-4">Configured Providers</h2>
        {providers.length === 0 ? (
          <p className="text-muted-foreground">No providers configured. Add one above or select a preset.</p>
        ) : (
          <div className="space-y-4">
            {providers.map((provider) => (
              <Card key={provider.id}>
                <CardContent className="pt-6">
                  <div className="flex justify-between items-start">
                    <div>
                      <h3 className="font-semibold">{provider.name}</h3>
                      <p className="text-sm text-muted-foreground">
                        {provider.provider_type} | {provider.default_model_id}
                      </p>
                      <p className="text-xs text-muted-foreground mt-1">{provider.base_url}</p>
                    </div>
                    <div className="flex items-center gap-2">
                      <span className={`px-2 py-1 text-xs rounded ${provider.is_enabled ? "bg-green-100" : "bg-gray-100"}`}>
                        {provider.is_enabled ? "Enabled" : "Disabled"}
                      </span>
                      <Button variant="outline" size="sm" onClick={() => deleteProvider(provider.id)}>
                        Delete
                      </Button>
                    </div>
                  </div>
                </CardContent>
              </Card>
            ))}
          </div>
        )}
      </section>
    </div>
  );
}