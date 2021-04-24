const cache: Record<string, Promise<void>> = {};

export default function (url: string) {
    if (url in cache) return cache[url];
    const promise = new Promise<void>((resolve, reject) => {
        const ele = document.createElement('script');
        ele.src = url;
        ele.onload = () => resolve();
        ele.onerror = () => {
            delete cache[url];
            reject(new Error(`Failed loading script at ${url}`));
        };
        document.head.appendChild(ele);
    });
    cache[url] = promise;
    return promise;
}
