import { BrowserRouter, Routes, Route } from "react-router-dom";
import { ProjectLayout } from "./components/layout/ProjectLayout";
import { ProvidersPage } from "./pages/ProvidersPage";
import { AgentsPage } from "./pages/AgentsPage";
import { ProjectsPage } from "./pages/ProjectsPage";
import { ProjectDetailPage } from "./pages/ProjectDetailPage";
import { WelcomePage } from "./pages/WelcomePage";

function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<WelcomePage />} />
        <Route path="/providers" element={<ProvidersPage />} />
        <Route path="/agents" element={<AgentsPage />} />
        <Route path="/projects" element={<ProjectsPage />} />
        <Route path="/project/:projectId" element={<ProjectLayout />}>
          <Route index element={<ProjectDetailPage />} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
}

export default App;