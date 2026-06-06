const DB_NAME = "launcherg-mobile-cache";
const STORE_NAME = "images";
const DB_VERSION = 1;

let dbPromise: Promise<IDBDatabase> | null = null;

export const initImageCacheDB = (): Promise<IDBDatabase> => {
  if (dbPromise) return dbPromise;

  dbPromise = new Promise((resolve, reject) => {
    const request = indexedDB.open(DB_NAME, DB_VERSION);

    request.onupgradeneeded = (event) => {
      const db = (event.target as IDBOpenDBRequest).result;
      if (!db.objectStoreNames.contains(STORE_NAME)) {
        db.createObjectStore(STORE_NAME, { keyPath: "path" });
      }
    };

    request.onsuccess = (event) => {
      resolve((event.target as IDBOpenDBRequest).result);
    };

    request.onerror = (event) => {
      reject((event.target as IDBOpenDBRequest).error);
    };
  });

  return dbPromise;
};

export const saveImageToCache = async (path: string, blob: Blob): Promise<void> => {
  try {
    const db = await initImageCacheDB();
    return new Promise((resolve, reject) => {
      const transaction = db.transaction(STORE_NAME, "readwrite");
      const store = transaction.objectStore(STORE_NAME);
      const request = store.put({ path, blob });

      request.onsuccess = () => resolve();
      request.onerror = () => reject(request.error);
    });
  } catch (e) {
    console.warn("Failed to save image to cache", e);
  }
};

export const getImageFromCache = async (path: string): Promise<Blob | null> => {
  try {
    const db = await initImageCacheDB();
    return new Promise((resolve, reject) => {
      const transaction = db.transaction(STORE_NAME, "readonly");
      const store = transaction.objectStore(STORE_NAME);
      const request = store.get(path);

      request.onsuccess = () => {
        const result = request.result;
        resolve(result ? result.blob : null);
      };
      request.onerror = () => reject(request.error);
    });
  } catch (e) {
    console.warn("Failed to get image from cache", e);
    return null;
  }
};

export const getAllCachedImages = async (): Promise<{ path: string; blob: Blob }[]> => {
  try {
    const db = await initImageCacheDB();
    return new Promise((resolve, reject) => {
      const transaction = db.transaction(STORE_NAME, "readonly");
      const store = transaction.objectStore(STORE_NAME);
      const request = store.getAll();

      request.onsuccess = () => {
        resolve(request.result || []);
      };
      request.onerror = () => reject(request.error);
    });
  } catch (e) {
    console.warn("Failed to get all cached images", e);
    return [];
  }
};
