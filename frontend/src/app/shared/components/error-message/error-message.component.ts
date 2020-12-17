import {Component, Input, OnInit} from '@angular/core';
import {ErrorMessage} from '../../../core/models';

@Component({
  selector: 'app-error-message',
  templateUrl: './error-message.component.html',
  styleUrls: ['./error-message.component.scss']
})
export class ErrorMessageComponent implements OnInit {

  @Input() errorMessage: ErrorMessage;

  constructor() {
  }

  ngOnInit(): void {
  }

}
