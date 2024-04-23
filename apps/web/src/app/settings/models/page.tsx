'use client'
import PageNav from '@/components/PageNav'
import Viewport from '@/components/Viewport'
import { ModelList } from '../_components/ModelList'

export default function ModelsSettings() {
  return (
    <Viewport.Page>
      <Viewport.Toolbar>
        <PageNav title="Settings" />
      </Viewport.Toolbar>
      <Viewport.Content className="p-6">
        <div className="h-10">Model Settings</div>
        <ModelList />
      </Viewport.Content>
    </Viewport.Page>
  )
}
