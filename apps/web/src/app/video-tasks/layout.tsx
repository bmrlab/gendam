import Sidebar from "@/components/Sidebar";

export default function Layout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <main className="min-h-screen flex">
      <Sidebar />
      <div className="flex-1">{children}</div>
    </main>
  );
}
