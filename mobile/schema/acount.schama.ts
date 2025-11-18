import { co } from 'jazz-tools';
import { ActivityLog } from './activity.schema';

const Root = co.map({
  activities: co.list(ActivityLog),
});

export const AppAccount = co.account({
  root: Root,
  profile: co.profile(),
});
