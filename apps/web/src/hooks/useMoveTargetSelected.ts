import { useExplorerContext } from '@/Explorer/hooks'
import { ExplorerItem } from '@/Explorer/types'
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
    async (target: ExplorerItem | null) => {
      for (let active of Array.from(explorer.selectedItems)) {
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
