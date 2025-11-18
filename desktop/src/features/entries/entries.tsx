import {useGetEntry} from "~/aether-sdk";

export const Entries = () => {
  const {data, isLoading, error} = useGetEntry();

  console.log(data, isLoading, error);

  return (
    <main className="bg-neutral-100 w-screen h-screen debug">
      <p className="text-2xl">Hello world, Inter from the year</p>
    </main>
  );
};
