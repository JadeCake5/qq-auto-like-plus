import { Outlet } from "react-router-dom";
import { SidebarNav } from "@/components/layout/SidebarNav";
import { StatusBar } from "@/components/layout/StatusBar";

export function Layout() {
  return (
    <div className="flex h-screen w-screen overflow-hidden bg-bg-base">
      <SidebarNav />
      <div className="flex flex-1 flex-col overflow-hidden">
        <StatusBar />
        <main className="flex-1 overflow-auto p-5">
          <Outlet />
        </main>
      </div>
    </div>
  );
}
