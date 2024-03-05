import Sidebar from "@/components/Sidebar";

export default function Layout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <main className="flex">
      <Sidebar />
      <div className="flex-1 min-h-screen bg-white">{children}</div>
    </main>
  );
}
