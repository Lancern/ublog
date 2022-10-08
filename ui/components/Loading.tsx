export default function Loading(): JSX.Element {
  return (
    <div className="flex justify-center py-32">
      <div className="animate-loading1 rounded-full bg-blue-600 dark:bg-blue-400 mx-8 h-4 w-4"></div>
      <div className="animate-loading2 rounded-full bg-violet-600 dark:bg-violet-400 mx-8 h-4 w-4"></div>
      <div className="animate-loading1 rounded-full bg-fuchsia-600 dark:bg-fuchsia-400 mx-8 h-4 w-4"></div>
    </div>
  );
}
