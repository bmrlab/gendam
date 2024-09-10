import { useCurrentLibrary } from '@/lib/library'
import { useEffect } from 'react'
import { TransformComponent, TransformWrapper, useControls } from 'react-zoom-pan-pinch'

function ViewerControls({ src }: { src: string }) {
  const { zoomToElement } = useControls()

  // every time the src changes, reset to default view
  useEffect(() => {
    zoomToElement('image-viewer', void 0, 0)
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [src])

  return <></>
}

export function BasicImageViewer({ src, alt }: { src: string; alt: string }) {
  return (
    <TransformWrapper
      onInit={(ref) => {
        ref.zoomToElement('image-viewer', void 0, 0)
      }}
      minScale={0.1}
      doubleClick={{
        mode: 'reset',
      }}
    >
      <>
        <ViewerControls src={src} />
        <TransformComponent
          wrapperStyle={{
            maxHeight: '100%',
            maxWidth: '100%',
            height: '100%',
            width: '100%',
            flexDirection: 'column',
            justifyContent: 'start',
          }}
        >
          {/* eslint-disable-next-line @next/next/no-img-element */}
          <img id="image-viewer" src={src} alt={alt} />
        </TransformComponent>
      </>
    </TransformWrapper>
  )
}

export default function ImageViewer({ hash, mimeType }: { hash: string; mimeType: string | null }) {
  const currentLibrary = useCurrentLibrary()

  // TODO 需要根据当前文件的类型来决定是否显示预览图（对于 image 无法原生显示的）
  return <BasicImageViewer src={currentLibrary.getFileSrc(hash)} alt={hash} />
}
