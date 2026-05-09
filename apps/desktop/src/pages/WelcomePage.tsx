export function WelcomePage() {
  return (
    <div className="flex-1 flex items-center justify-center">
      <div className="text-center space-y-4">
        <h1 className="text-4xl font-bold">SuperCompany Coding</h1>
        <p className="text-muted-foreground text-lg">AI-powered multi-agent coding workspace</p>
        <div className="flex gap-4 justify-center pt-8">
          <a href="/providers" className="px-4 py-2 bg-primary text-primary-foreground rounded-lg hover:bg-primary/90">
            Configure Providers
          </a>
          <a href="/projects" className="px-4 py-2 border border-input rounded-lg hover:bg-accent">
            New Project
          </a>
        </div>
      </div>
    </div>
  );
}