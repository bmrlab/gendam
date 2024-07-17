'use client'
import { useExplorerContext, useExplorerViewContext } from '@/Explorer/hooks'
import PageNav from '@/components/PageNav'
import Viewport from '@/components/Viewport'
// import { rspc } from '@/lib/rspc'
import { useInspector } from '@/components/Inspector/store'
import Icon from '@gendam/ui/icons'
import { Button } from '@gendam/ui/v2/button'
import classNames from 'classnames'
import { useRouter } from 'next/navigation'
import { useCallback, useRef } from 'react'
import SearchForm, { type SearchFormRef } from '../../search/SearchForm' // TODO: 这样不大好，应该是一个公共组件
import TitleDialog from './TitleDialog'

export default function Header() {
  const router = useRouter()
  const explorer = useExplorerContext()
  const ExplorerView = useExplorerViewContext()
  const inspector = useInspector()

  const searchFormRef = useRef<SearchFormRef>(null)
  // const [searchPayload, setSearchPayload] = useState<SearchRequestPayload | null>(null)
  const onSearchFormSubmit = useCallback(() => {
    if (searchFormRef.current) {
      const search = new URLSearchParams()
      const value = searchFormRef.current.getValue()
      if (value) {
        search.set('text', value.text)
        search.set('recordType', value.recordType)
        router.push(`/search?${search}`)
      }
    }
  }, [router])

  return (
    <>
      <Viewport.Toolbar className="relative">
        <PageNav title={explorer.materializedPath === '/' ? 'Library' : explorer.materializedPath} />
        <div className="absolute left-1/3 w-1/3">
          <SearchForm ref={searchFormRef} onSubmit={() => onSearchFormSubmit()} />
        </div>
        <div className="ml-auto"></div>
        <div className="text-ink/70 flex items-center gap-1 justify-self-end">
          {ExplorerView.headerTools}
          {!!ExplorerView.headerTools && <div className="bg-toolbar-line mx-1 h-4 w-px"></div>}

          <Button
            variant="ghost"
            size="sm"
            className={classNames(
              'h-7 w-7 p-1 transition-none',
              explorer.settings.layout === 'grid' && 'bg-toolbar-hover',
            )}
            onClick={() => explorer.settings.update({ layout: 'grid' })}
          >
            <Icon.Grid className="size-4" />
          </Button>
          <Button
            variant="ghost"
            size="sm"
            className={classNames(
              'h-7 w-7 p-1 transition-none',
              explorer.settings.layout === 'list' && 'bg-toolbar-hover',
            )}
            onClick={() => explorer.settings.update({ layout: 'list' })}
          >
            <Icon.List className="size-4" />
          </Button>
          <Button
            variant="ghost"
            size="sm"
            className={classNames(
              'h-7 w-7 p-1 transition-none',
              explorer.settings.layout === 'media' && 'bg-toolbar-hover',
            )}
            onClick={() => explorer.settings.update({ layout: 'media' })}
          >
            <Icon.SelfAdapting className="size-4" />
          </Button>

          <div className="bg-toolbar-line mx-1 h-4 w-px"></div>

          <Button
            variant="ghost"
            size="sm"
            className={classNames('h-7 w-7 p-1 transition-none', inspector.show && 'bg-toolbar-hover')}
            onClick={() => inspector.setShow(!inspector.show)}
          >
            <Icon.Sidebar className="size-4" />
          </Button>
        </div>
      </Viewport.Toolbar>
      <TitleDialog />
    </>
  )
}
