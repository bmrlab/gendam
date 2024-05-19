import { useExplorerContext } from '@/Explorer/hooks'
import { type FilePath } from '@/lib/bindings'
import { type ExplorerItem } from '@/Explorer/types'
import { queryClient, rspc } from '@/lib/rspc'
import { useSearchParams } from 'next/navigation'
import { useCallback, useMemo } from 'react'

export const useMoveTargetSelected = () => {
  const searchParams = useSearchParams()
  let dirInSearchParams = searchParams.get('dir') || '/'
  if (!/^\/([^/\\:*?"<>|]+\/)+$/.test(dirInSearchParams)) {
    dirInSearchParams = '/'
  }
  // currentPath 必须以 / 结尾, 调用 setCurrentPath 的地方自行确保格式正确
  // const [materializedPath, setMaterializedPath] = useState<string>(dirInSearchParams)
  const materializedPath = useMemo(() => dirInSearchParams, [dirInSearchParams])
  const moveMut = rspc.useMutation(['assets.move_file_path'])

  const explorer = useExplorerContext()

  const onMoveTargetSelected = useCallback(
    async (target: FilePath | null) => {
      // 目前只允许 FilePath 数据被移动，搜索结果等不能被移动
      type T = Extract<ExplorerItem, { type: 'FilePath' }>
      const selected = Array.from(explorer.selectedItems).filter((item) => item.type === 'FilePath') as T[]
      for (let active of selected.map(item => item.filePath)) {
        // target 可以为空，为空就是根目录，这时候不需要检查 target.id !== active.id，因为根目录本身不会被移动
        if (target && target.id === active.id) {
          continue
        }
        try {
          await moveMut.mutateAsync({
            active: {
              id: active.id,
              materializedPath: active.materializedPath,
              isDir: active.isDir,
              name: active.name,
            },
            target: target
              ? {
                  id: target.id,
                  materializedPath: target.materializedPath,
                  isDir: target.isDir,
                  name: target.name,
                }
              : null,
          })
        } catch (error) {}
        queryClient.invalidateQueries({
          queryKey: ['assets.list', { materializedPath: materializedPath }],
        })
        queryClient.invalidateQueries({
          queryKey: [
            'assets.list',
            {
              materializedPath: target ? target.materializedPath + target.name + '/' : '/',
            },
          ],
        })
      }
    },
    [explorer.selectedItems, materializedPath, moveMut],
  )
  return {
    onMoveTargetSelected,
  }
}
