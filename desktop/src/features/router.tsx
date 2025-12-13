import {
	createBrowserRouter,
	createRoutesFromElements,
	Route,
} from "react-router";
import { Journal } from "./journal/journal";

export const router = createBrowserRouter(
	createRoutesFromElements(<Route path="" element={<Journal />} />),
);
