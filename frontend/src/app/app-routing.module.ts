import {NgModule} from '@angular/core';
import {RouterModule, Routes} from '@angular/router';
import {NavigationComponent} from './navigation/components/navigation/navigation.component';

const routes: Routes = [
  {path: '', redirectTo: 'user/profile', pathMatch: 'full'},
  {
    path: '', component: NavigationComponent, children: [
      {
        path: 'user',
        loadChildren: () => import('src/app/user/user.module').then(m => m.UserModule)
      },
      {
        path: 'experiment',
        loadChildren: () => import('src/app/experiment/experiment.module').then(m => m.ExperimentModule)
      },
    ]
  }
];

@NgModule({
  imports: [RouterModule.forRoot(routes)],
  exports: [RouterModule]
})
export class AppRoutingModule {
}
