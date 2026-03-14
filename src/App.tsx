import { Routes, Route, Navigate } from "react-router-dom";
import { TooltipProvider } from "@/components/ui/tooltip";
import { Toaster } from "@/components/ui/sonner";
import { Layout } from "@/components/layout/Layout";
import { TauriEventProvider } from "@/components/TauriEventProvider";
import Dashboard from "@/pages/Dashboard";
import Friends from "@/pages/Friends";
import Statistics from "@/pages/Statistics";
import Logs from "@/pages/Logs";
import Settings from "@/pages/Settings";

function App() {
  return (
    <TooltipProvider delay={500}>
      <TauriEventProvider />
      <Routes>
        <Route element={<Layout />}>
          <Route path="/dashboard" element={<Dashboard />} />
          <Route path="/friends" element={<Friends />} />
          <Route path="/statistics" element={<Statistics />} />
          <Route path="/logs" element={<Logs />} />
          <Route path="/settings" element={<Settings />} />
          <Route path="*" element={<Navigate to="/dashboard" replace />} />
        </Route>
      </Routes>
      <Toaster
        position="bottom-right"
        richColors
        toastOptions={{
          className:
            "!bg-bg-elevated !border-border !text-text-primary",
        }}
      />
    </TooltipProvider>
  );
}

export default App;
