export interface User {
  id: number;
  username: string;
  email: string;
  role: "Admin" | "Editor" | "Viewer";
  farm_id: number;
}
export interface Session {
  user: User;
  farm_name: string;
}
