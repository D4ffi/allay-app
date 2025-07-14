import {AllayLayout} from "../components/common/AllayLayout.tsx";
import {ActionBar} from "../components/common/ActionBar.tsx";

const Home = () => {
  return (
    <div className="h-screen pt-12">
      <AllayLayout />
      <ActionBar />

        <div className="flex flex-col justify-center items-center h-full gap-4 opacity-30">
            <img src="/profile-off.png" alt="Allay Off Icon" className="w-30 h-30 drop-shadow-lg drop-shadow-gray-900"/>
            <p className="text-center text-balance">
                No server's saved, to create a new one,<br />
                press the + button in the action bar menu.
            </p>


        </div>

    </div>
  );
};

export default Home;