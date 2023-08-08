import { Link } from "react-router-dom";

export default function Navbar() {
  return (
    <nav>
      <ul>
        <Link href={"/"}>
          <li>Home</li>
        </Link>
        <Link href={"/contact"}>
          <li>Contact</li>
        </Link>
      </ul>
    </nav>
  );
}
