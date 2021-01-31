import {Component, OnInit} from '@angular/core';
import {MainComponent} from '../../../shared/components/main/main.component';
import {Job, SlimRunner} from '../../models';
import {ExperimentViewModelService} from '../../services/experiment-view-model.service';
import {ActivatedRoute} from '@angular/router';
import {switchMap} from 'rxjs/operators';

@Component({
  selector: 'app-job',
  templateUrl: './job.component.html',
  styleUrls: ['./job.component.scss']
})
export class JobComponent extends MainComponent implements OnInit {

  job: Job;
  runner: SlimRunner;

  get isPageReady(): boolean {
    return !!this.job && !!this.runner;
  }

  constructor(
    private viewModel: ExperimentViewModelService,
    private activatedRoute: ActivatedRoute
  ) {
    super();
  }

  ngOnInit(): void {
    this.subs.add(
      this.activatedRoute.params.pipe(
        switchMap(params => this.viewModel.job(params.id))
      ).subscribe(([job, runner]) => {
        this.job = job;
        this.runner = runner;
      })
    );
  }

}
