import {NgModule} from '@angular/core';
import {CommonModule} from '@angular/common';
import {NavigationComponent} from './components/navigation/navigation.component';
import {SharedModule} from '../shared/shared.module';
import {CoreModule} from '../core/core.module';


@NgModule({
  declarations: [NavigationComponent],
  imports: [
    CommonModule,
    SharedModule,
    CoreModule
  ],
  exports: [
    NavigationComponent
  ]
})
export class NavigationModule {
}
