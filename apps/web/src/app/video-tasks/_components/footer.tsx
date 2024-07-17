'use client'
import Viewport from '@/components/Viewport'
import { useBoundStore } from '../_store'
import { useTranslation } from 'react-i18next'

export default function TaskFooter({ total }: { total: number }) {
  const selected = useBoundStore.use.videoSelected()
  const { t } = useTranslation()

  return (
    <Viewport.StatusBar className="justify-center">
      <div className="text-xs text-neutral-500">
        {t('task.footer', {len: selected.length ?? 0, total})}
      </div>
    </Viewport.StatusBar>
  )
}
