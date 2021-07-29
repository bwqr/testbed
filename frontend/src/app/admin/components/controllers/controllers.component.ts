import {Component, OnInit} from '@angular/core';
import {ExperimentViewModelService} from '../../../experiment/services/experiment-view-model.service';
import {MainComponent} from '../../../shared/components/main/main.component';
import {SlimController} from '../../../experiment/models';
import {formats} from '../../../../defs';

@Component({
  selector: 'app-controllers',
  templateUrl: './controllers.component.html',
  styleUrls: ['./controllers.component.scss']
})
export class ControllersComponent extends MainComponent implements OnInit {

  controllers: SlimController[];

  formats = formats;

  get isPageReady(): boolean {
    return !!this.controllers;
  }

  constructor(
    private experimentViewModel: ExperimentViewModelService
  ) {
    super();
  }

  ngOnInit(): void {
    this.subs.add(
      this.experimentViewModel.controllers().subscribe(controllers => this.controllers = controllers)
    );
  }

}
