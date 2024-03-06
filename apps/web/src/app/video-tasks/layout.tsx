import Sidebar from "@/components/Sidebar";
import ClientLayout from "@/components/ClientLayout";

export default function Layout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <ClientLayout>
      <main className="flex">
        <Sidebar />
        <div className="flex-1 min-h-screen bg-white">{children}</div>
      </main>
    </ClientLayout>
  );
}
