import { co } from 'jazz-tools';
import { Activity } from './activity.schema';

const Root = co.map({
  activities: co.list(Activity),
});

export const AppAccount = co.account({
  root: Root,
  profile: co.profile(),
});
