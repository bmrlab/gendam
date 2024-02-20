import { convertFileSrc } from '@tauri-apps/api/tauri';
import { open } from '@tauri-apps/api/dialog';

// export const getContentUrl = (contentPath: string): string => {
//   /**
//    * 返回 DAM 中一个内容的 URL, 当前版本里面是一个本地的 URL, 可能出现在本地磁盘任何地方
//    * TODO: axum 启动了一个 serve 静态文件的服务, 这个方法需要实现一个 tauri 环境下的版本
//    */
//   return `http://localhost:3001/contents/${contentPath}`
// };

// export const getArtifactUrl = (artifactPath: string): string => {
//   /**
//    * 返回内容处理中间结果的素材 URL, 存在 local_data_dir/file_hash 下面
//    * TODO: axum 启动了一个 serve 静态文件的服务, 这个方法需要实现一个 tauri 环境下的版本
//    */
//   return `http://localhost:3001/artifacts/${artifactPath}`
// };

export const getLocalFileUrl = (fileFullPath: string): string => {
  if (typeof window !== 'undefined' && typeof window.__TAURI__ !== 'undefined') {
    return convertFileSrc(fileFullPath);
  } else {
    return `http://localhost:3001/file/localhost/${fileFullPath}`
  }
};

export const selectFile = async (): Promise<string | null> => {
  if (typeof window !== 'undefined' && typeof window.__TAURI__ !== 'undefined') {
    const result = await open({ directory: false });
    if (result) {
      return result as string;
    } else {
      return null;
    }
  } else {
    // 网页版无法使用 js 选择文件, 这里要改成做一个 FileSelect 组件, 然后在页面中引入
    return null;
  }
};
