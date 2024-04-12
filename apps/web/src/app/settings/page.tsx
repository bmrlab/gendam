'use client'
import PageNav from '@/components/PageNav'
import Viewport from '@/components/Viewport'

export default function Settings() {
  return (
    <Viewport.Page>
      <Viewport.Toolbar>
        <PageNav title="Settings" />
      </Viewport.Toolbar>
      <Viewport.Content className="p-6">
        <div className="h-10">
          用户 / login / logout
        </div>
        <div className="h-px bg-app-line my-8"></div>
        <div className="h-10">
          Library / reload workspace
        </div>
        <div className="h-px bg-app-line my-8"></div>
        <div className="h-10">
          模型设置
        </div>
      </Viewport.Content>
    </Viewport.Page>
  )
}
