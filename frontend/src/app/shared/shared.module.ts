import {NgModule} from '@angular/core';
import {CommonModule} from '@angular/common';
import {MainComponent} from './components/main/main.component';
import {ReactiveFormsModule} from '@angular/forms';
import {HttpClientModule} from '@angular/common/http';
import { ErrorMessageComponent } from './components/error-message/error-message.component';


@NgModule({
  declarations: [
    MainComponent,
    ErrorMessageComponent
  ],
  imports: [
    CommonModule,
  ],
  exports: [
    ReactiveFormsModule,
    HttpClientModule,
    ErrorMessageComponent
  ]
})
export class SharedModule {
}
