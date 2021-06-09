import {NgModule} from '@angular/core';
import {PreloadAllModules, RouterModule, Routes} from '@angular/router';
import {NavigationComponent} from './navigation/components/navigation/navigation.component';
import {UserResolveService} from './user/services/user-resolve.service';

const routes: Routes = [
  {path: '', redirectTo: 'user/profile', pathMatch: 'full'},
  {
    path: '', component: NavigationComponent, resolve: {user: UserResolveService},
    children: [
      {
        path: 'user',
        loadChildren: () => import('src/app/user/user.module').then(m => m.UserModule)
      },
      {
        path: 'experiment',
        loadChildren: () => import('src/app/experiment/experiment.module').then(m => m.ExperimentModule)
      },
      {
        path: 'admin',
        loadChildren: () => import('src/app/admin/admin.module').then(m => m.AdminModule)
      },
      {
        path: 'slot',
        loadChildren: () => import('src/app/slot/slot.module').then(m => m.SlotModule)
      }
    ]
  }
];

@NgModule({
  imports: [RouterModule.forRoot(routes, {
    preloadingStrategy: PreloadAllModules
  })],
  exports: [RouterModule]
})
export class AppRoutingModule {
}
